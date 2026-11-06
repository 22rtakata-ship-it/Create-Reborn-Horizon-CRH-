// Copyright (c) The Create Team / The Creators of Create
// Copyright (c) 2026 create-horizon contributors
// SPDX-License-Identifier: MIT

// ChestMountedStorage and the exposed MountedStorageManager item handler.

use super::{Game, create_port_contraption as contraption};
use crate::{containers::BlockEntity, items::ItemStack};
use create_engine::{
    contraption::QuarterTurn,
    kinetics::{Axis, Pos},
};
use horizon_client_render::create_contraption_scene::{MAX_RECORDS, Record, StrokeKind};
use horizon_items::item_nbt::{encode_items, item_from_nbt};
use horizon_world::{
    attachments::{AttachmentCodec, AttachmentError, AttachmentType, Attachments, Holder},
    chunk::BlockState,
    nbt::{Compound, Limits, Tag, write_network},
    registry::BlockStateId,
};
use std::collections::BTreeMap;

const BLOCK: &str = "minecraft:chest";
const SIZE: usize = 27;
const SLOT_LIMIT: u32 = 64;
const MAX_STORAGE_BYTES: usize = 1024 * 1024;

#[derive(Clone, Debug)]
pub(super) struct Chest {
    local: Pos,
    slots: Vec<Option<ItemStack>>,
}

type Store = BTreeMap<Pos, Chest>;
type Edit = (Pos, BlockStateId, BlockStateId);

#[derive(Clone, Copy)]
pub(super) struct Mount {
    controller: Pos,
    local: Pos,
}

// Kept in the same saved chunk as the pose. Unlike the scene attachment,
// inventories must never be sent to tracking clients in chunk updates.
const STORAGE: AttachmentType<Store> = AttachmentType {
    namespace: "create",
    name: "mounted_item_storage",
    default: Store::new,
    codec: Some(AttachmentCodec { encode, decode }),
    copy_on_death: false,
    sync: false,
};

pub(super) fn chest_state(state: &BlockState) -> bool {
    state.name == BLOCK
        && state.properties.len() == 3
        && state
            .properties
            .get("type")
            .is_some_and(|kind| kind == "single")
        && state
            .properties
            .get("waterlogged")
            .is_some_and(|wet| wet == "false")
        && state
            .properties
            .get("facing")
            .is_some_and(|face| matches!(face.as_str(), "north" | "south" | "east" | "west"))
}

pub(super) fn can_capture(game: &Game<'_>, position: Pos, state: &BlockState) -> bool {
    let key = (position[0], position[1], position[2]);
    if !chest_state(state)
        || !matches!(game.world.block_entities.get(&key), Some(BlockEntity::Chest(slots)) if slots.len() == SIZE)
        || game.world.block_names.contains_key(&key)
        || game.world.block_lock(key).is_some()
        || game.container == Some(key)
        || game.lan.as_ref().is_some_and(|host| {
            host.hub.guests.iter().any(|guest| {
                guest.dimension == game.dim.id()
                    && guest.menu.as_ref().is_some_and(|menu| match &menu.slots {
                        crate::lan::menu::Slots::Block(pos) => *pos == key,
                        crate::lan::menu::Slots::Double(first, second) => {
                            *first == key || *second == key
                        }
                        _ => false,
                    })
            })
        })
    {
        return false;
    }
    let column = (position[0] >> 4, position[2] >> 4);
    if game
        .world
        .block_entities
        .chunk_attachments(column.0, column.1)
        .iter()
        .any(|(pos, _)| *pos == key)
    {
        return false;
    }
    let Some(saved) = game
        .world
        .chunks
        .get(&column)
        .and_then(|chunk| chunk.saved.as_ref())
    else {
        return true;
    };
    !saved.rolled_loot.iter().any(|loot| loot.pos == key)
        && !saved
            .block_entity_attachments
            .iter()
            .any(|(pos, _)| *pos == key)
        && saved
            .block_entity_sources
            .iter()
            .find(|(pos, _)| *pos == key)
            .is_none_or(|(_, tag)| plain_source(tag))
}

fn plain_source(tag: &Tag) -> bool {
    let Tag::Compound(fields) = tag else {
        return false;
    };
    fields
        .iter()
        .all(|(key, value)| match key.to_string().as_deref() {
            Ok("id") => *value == Tag::String(BLOCK.into()),
            Ok("x" | "y" | "z") => matches!(value, Tag::Int(_)),
            Ok("keepPacked") => matches!(value, Tag::Byte(0)),
            Ok("components") => matches!(value, Tag::Compound(components) if components.is_empty()),
            Ok("Items") => decode_slots(value).is_some(),
            _ => false,
        })
}

fn local_chest(blocks: &[(Pos, BlockState)]) -> Result<Option<Pos>, AttachmentError> {
    let mut result = None;
    for (position, block) in blocks.iter().filter(|(_, block)| block.name == BLOCK) {
        if !chest_state(block) || !valid_local(*position) || result.replace(*position).is_some() {
            return Err(AttachmentError::Decode);
        }
    }
    Ok(result)
}

fn valid_local(pos: Pos) -> bool {
    pos.iter().all(|component| component.unsigned_abs() <= 15)
}

pub(super) fn mounted_chest(record: &Record) -> Option<Mount> {
    let supported = match record.stroke_kind() {
        Some(StrokeKind::Gantry | StrokeKind::RopePulley | StrokeKind::Piston) => true,
        None => record.axis == Axis::Y.index(),
    };
    if !supported || record.clock_hand.is_some() || record.return_home {
        return None;
    }
    Some(Mount {
        controller: record.controller,
        local: local_chest(&record.blocks).ok()??,
    })
}

pub(super) fn capture(
    game: &Game<'_>,
    anchor: Pos,
    blocks: &[(Pos, BlockState)],
) -> Result<Option<Chest>, AttachmentError> {
    let Some(local) = local_chest(blocks)? else {
        return Ok(None);
    };
    let position = place(anchor, local).ok_or(AttachmentError::Encode)?;
    let state = game
        .pack
        .assets
        .registry
        .state(contraption::state(game, position))
        .ok_or(AttachmentError::Encode)?;
    if !can_capture(game, position, state) {
        return Err(AttachmentError::Encode);
    }
    let Some(BlockEntity::Chest(slots)) =
        game.world
            .block_entities
            .get(&(position[0], position[1], position[2]))
    else {
        return Err(AttachmentError::Encode);
    };
    checked_slots(slots)?;
    Ok(Some(Chest {
        local,
        slots: slots.clone(),
    }))
}

pub(super) fn commit_capture(
    game: &mut Game<'_>,
    key: [i32; 2],
    records: Vec<Record>,
    edits: &[Edit],
    controller: Pos,
    chest: Option<Chest>,
) -> bool {
    let Some(chest) = chest else {
        return contraption::commit(game, key, records, edits);
    };
    let Ok((mut store, before)) = private_map(game, key) else {
        return false;
    };
    if store.insert(controller, chest).is_some() || write_store(game, key, store).is_err() {
        return false;
    }
    commit(game, key, records, edits, before)
}

pub(super) fn commit_landing(
    game: &mut Game<'_>,
    key: [i32; 2],
    records: Vec<Record>,
    edits: &[Edit],
    record: &Record,
    origin: Pos,
    turn: QuarterTurn,
) -> bool {
    let local = match local_chest(&record.blocks) {
        Ok(Some(local)) => local,
        Ok(None) => return contraption::commit(game, key, records, edits),
        Err(_) => return false,
    };
    let Ok((mut store, before)) = private_map(game, key) else {
        return false;
    };
    let Some(chest) = store
        .remove(&record.controller)
        .filter(|chest| chest.local == local)
    else {
        return false;
    };
    let Some(position) = turn.place(origin, chest.local) else {
        return false;
    };
    let target = (position[0], position[1], position[2]);
    if contraption::column(position) != key
        || game.world.block_entities.contains_key(&target)
        || !edits.iter().any(|(p, _, after)| {
            *p == position
                && game
                    .pack
                    .assets
                    .registry
                    .state(*after)
                    .is_some_and(chest_state)
        })
        || write_store(game, key, store).is_err()
    {
        return false;
    }
    if !commit(game, key, records, edits, before) {
        return false;
    }
    // The world edit creates a vanilla chest; install its already-validated
    // contents before the server can save, tick hoppers, or accept a click.
    game.world
        .block_entities
        .insert(target, BlockEntity::Chest(chest.slots));
    game.native_block_entity_sync(position);
    true
}

fn place(anchor: Pos, local: Pos) -> Option<Pos> {
    Some([
        anchor[0].checked_add(local[0])?,
        anchor[1].checked_add(local[1])?,
        anchor[2].checked_add(local[2])?,
    ])
}

fn private_map(
    game: &mut Game<'_>,
    key: [i32; 2],
) -> Result<(Store, Attachments), AttachmentError> {
    let attachments = game.attachment_map_quiet(Holder::Chunk(key))?;
    let store = attachments.get_existing(&STORAGE)?.unwrap_or_default();
    Ok((store, attachments.clone()))
}

fn write_store(game: &mut Game<'_>, key: [i32; 2], store: Store) -> Result<(), AttachmentError> {
    let attachments = game.attachment_map_quiet(Holder::Chunk(key))?;
    if store.is_empty() {
        attachments.remove(&STORAGE)?;
        Ok(())
    } else {
        attachments.set(&STORAGE, store)
    }
}

fn commit(
    game: &mut Game<'_>,
    key: [i32; 2],
    records: Vec<Record>,
    edits: &[Edit],
    before: Attachments,
) -> bool {
    if contraption::commit(game, key, records, edits) {
        return true;
    }
    restore_map(game, key, before);
    false
}

fn restore_map(game: &mut Game<'_>, key: [i32; 2], before: Attachments) {
    if let Ok(map) = game.attachment_map_quiet(Holder::Chunk(key)) {
        *map = before;
    }
}

/// ItemHelper.extract(EXACTLY, 1) scans the mounted handler in slot order.
/// Publish the withdrawn item only if its recipient commits in the same column.
pub(super) fn withdraw_one(
    game: &mut Game<'_>,
    mount: Mount,
    matches_filter: impl FnMut(&ItemStack) -> bool,
    receive: impl FnOnce(&mut Game<'_>, &ItemStack) -> bool,
) -> Option<ItemStack> {
    withdraw(
        game,
        mount,
        |_, slots| extract_one(slots, matches_filter),
        receive,
    )
}

pub(super) fn withdraw_for_chute(
    game: &mut Game<'_>,
    mount: Mount,
    receive: impl FnOnce(&mut Game<'_>, &ItemStack) -> bool,
) -> Option<ItemStack> {
    use create_engine::chute::{Source, pull_plan};

    withdraw(
        game,
        mount,
        |game, slots| {
            let candidates = slots.iter().enumerate().filter_map(|(slot, item)| {
                let stack = item.as_ref()?;
                Some(Source {
                    slot,
                    stack,
                    count: stack.count,
                    maximum: super::create_port::stack_maximum(game, stack),
                })
            });
            let plan = pull_plan(candidates, ItemStack::same_item_same_components);
            let mut result: Option<ItemStack> = None;
            for (slot, count) in plan {
                let item = take_from_slot(slots.get_mut(slot)?, count)?;
                if let Some(result) = &mut result {
                    result.count += item.count;
                } else {
                    result = Some(item);
                }
            }
            result
        },
        receive,
    )
}

fn withdraw(
    game: &mut Game<'_>,
    mount: Mount,
    extract: impl FnOnce(&Game<'_>, &mut [Option<ItemStack>]) -> Option<ItemStack>,
    receive: impl FnOnce(&mut Game<'_>, &ItemStack) -> bool,
) -> Option<ItemStack> {
    let key = contraption::column(mount.controller);
    let holder = Holder::Chunk(key);
    let mut store = game
        .attachment_map_quiet(holder)
        .ok()?
        .get_existing(&STORAGE)
        .ok()??;
    let chest = store
        .get_mut(&mount.controller)
        .filter(|chest| chest.local == mount.local)?;
    let item = extract(game, &mut chest.slots)?;
    let before = game.attachment_map_quiet(holder).ok()?.clone();
    write_store(game, key, store).ok()?;
    if receive(game, &item) {
        Some(item)
    } else {
        restore_map(game, key, before);
        None
    }
}

fn extract_one(
    slots: &mut [Option<ItemStack>],
    mut matches_filter: impl FnMut(&ItemStack) -> bool,
) -> Option<ItemStack> {
    let slot = slots.iter_mut().find(|slot| {
        slot.as_ref()
            .is_some_and(|stack| stack.count > 0 && matches_filter(stack))
    })?;
    take_from_slot(slot, 1)
}

fn take_from_slot(slot: &mut Option<ItemStack>, count: u32) -> Option<ItemStack> {
    let stack = slot.as_mut()?;
    let mut item = stack.clone();
    item.count = count.min(stack.count);
    stack.count -= item.count;
    if stack.count == 0 {
        *slot = None;
    }
    Some(item)
}

/// Insert a block's drops in their loot-table order. Suppress accepted stacks
/// only when their new inventory snapshot was saved successfully.
pub(super) fn deposit(game: &mut Game<'_>, mount: Mount, source: Pos, drops: &mut [ItemStack]) {
    let key = contraption::column(mount.controller);
    // The source block and the inventory must persist in one chunk snapshot.
    if contraption::column(source) != key
        || drops.is_empty()
        || !game
            .world
            .chunks
            .get(&(key[0], key[1]))
            .and_then(|chunk| chunk.saved.as_ref())
            .is_some_and(|saved| saved.attachments.has(&STORAGE))
    {
        return;
    }
    let Ok(Some(mut store)) = game
        .attachment_map_quiet(Holder::Chunk(key))
        .and_then(|map| map.get_existing(&STORAGE))
    else {
        return;
    };
    let Some(chest) = store
        .get_mut(&mount.controller)
        .filter(|chest| chest.local == mount.local)
    else {
        return;
    };
    let mut accepted = Vec::with_capacity(drops.len());
    for stack in drops.iter() {
        let limit = match stack
            .data()
            .and_then(|data| data.other("minecraft:max_stack_size"))
        {
            Some(Tag::Int(limit @ 1..=99)) => *limit as u32,
            _ => game
                .items()
                .get(&stack.item)
                .map_or(0, |item| item.max_stack),
        };
        accepted.push(insert(&mut chest.slots, stack, limit.min(SLOT_LIMIT)));
    }
    if accepted.iter().all(|&count| count == 0) || write_store(game, key, store).is_err() {
        return;
    }
    for (stack, count) in drops.iter_mut().zip(accepted) {
        stack.count -= count;
    }
}

fn insert(slots: &mut [Option<ItemStack>], stack: &ItemStack, limit: u32) -> u32 {
    let mut remaining = stack.count;
    if limit == 0 || stack.item.as_ref() == "minecraft:air" {
        return 0;
    }
    for slot in slots {
        if remaining == 0 {
            break;
        }
        match slot {
            Some(existing) if existing.same_item_same_components(stack) => {
                let amount = limit.saturating_sub(existing.count).min(remaining);
                existing.count += amount;
                remaining -= amount;
            }
            None => {
                let amount = limit.min(remaining);
                let mut stored = stack.clone();
                stored.count = amount;
                *slot = Some(stored);
                remaining -= amount;
            }
            _ => {}
        }
    }
    stack.count - remaining
}

fn encode(store: &Store) -> Result<Compound, AttachmentError> {
    if store.len() > MAX_RECORDS || store.values().any(|chest| !valid_local(chest.local)) {
        return Err(AttachmentError::Encode);
    }
    let entries = store
        .iter()
        .map(|(controller, chest)| {
            Ok(Tag::compound([
                ("Controller", Tag::IntArray(controller.to_vec())),
                ("Pos", Tag::IntArray(chest.local.to_vec())),
                ("Items", checked_slots(&chest.slots)?),
            ]))
        })
        .collect::<Result<Vec<_>, AttachmentError>>()?;
    Ok(Compound::from([
        ("Version".into(), Tag::Int(1)),
        ("Chests".into(), Tag::List(entries)),
    ]))
}

fn checked_slots(slots: &[Option<ItemStack>]) -> Result<Tag, AttachmentError> {
    let nbt = encode_items(slots);
    let limits = Limits {
        max_bytes: MAX_STORAGE_BYTES,
        ..Limits::default()
    };
    if slots.len() != SIZE
        || write_network(&nbt, limits).is_err()
        || !decode_slots(&nbt).is_some_and(|decoded| decoded == slots)
    {
        return Err(AttachmentError::Encode);
    }
    Ok(nbt)
}

fn decode(saved: &Compound) -> Result<Store, AttachmentError> {
    let invalid = AttachmentError::Decode;
    let Some(Tag::List(entries)) = saved.get(&"Chests".into()) else {
        return Err(invalid);
    };
    if saved.len() != 2
        || saved.get(&"Version".into()) != Some(&Tag::Int(1))
        || entries.len() > MAX_RECORDS
    {
        return Err(invalid);
    }
    let position = |entry: &Tag, key: &str| match entry.get(key) {
        Some(Tag::IntArray(values)) => <Pos>::try_from(values.as_slice()).map_err(|_| invalid),
        _ => Err(invalid),
    };
    let mut store = Store::new();
    for entry in entries {
        if !matches!(entry, Tag::Compound(fields) if fields.len() == 3) {
            return Err(invalid);
        }
        let controller = position(entry, "Controller")?;
        let local = position(entry, "Pos")?;
        let items = entry.get("Items").ok_or(invalid)?;
        let slots = decode_slots(items).ok_or(invalid)?;
        let limits = Limits {
            max_bytes: MAX_STORAGE_BYTES,
            ..Limits::default()
        };
        if !valid_local(local)
            || write_network(items, limits).is_err()
            || store.insert(controller, Chest { local, slots }).is_some()
        {
            return Err(invalid);
        }
    }
    Ok(store)
}

fn decode_slots(tag: &Tag) -> Option<Vec<Option<ItemStack>>> {
    let Tag::List(items) = tag else { return None };
    if items.len() > SIZE {
        return None;
    }
    let mut slots = vec![None; SIZE];
    for item in items {
        let Some(Tag::Byte(slot @ 0..=26)) = item.get("Slot") else {
            return None;
        };
        let target = &mut slots[*slot as usize];
        if target.is_some() {
            return None;
        }
        *target = Some(item_from_nbt(item)?);
    }
    // Horizon's vanilla decoder tolerates malformed counts and unknown
    // fields. Keep the mounted codec strict so nothing is silently altered.
    (encode_items(&slots) == *tag).then_some(slots)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mounted_withdrawal_takes_one_from_first_matching_slot() {
        let sapling = "minecraft:oak_sapling";
        let filter = |stack: &ItemStack| stack.item.as_ref() == sapling;
        let mut slots = vec![None; SIZE];
        slots[0] = Some(ItemStack::new("minecraft:oak_log", 4));
        slots[3] = Some(ItemStack::new(sapling, 2));
        slots[8] = Some(ItemStack::new(sapling, 1));

        for index in [3, 3, 8] {
            let before = slots[index].as_ref().unwrap().count;
            assert_eq!(
                extract_one(&mut slots, filter),
                Some(ItemStack::new(sapling, 1))
            );
            assert_eq!(
                slots[index].as_ref().map_or(0, |stack| stack.count),
                before - 1
            );
        }
        assert_eq!(extract_one(&mut slots, filter), None);
        assert_eq!(slots[0], Some(ItemStack::new("minecraft:oak_log", 4)));
        assert_eq!(
            extract_one(&mut slots, |_| true),
            Some(ItemStack::new("minecraft:oak_log", 1))
        );
    }

    #[test]
    fn moving_storage_uses_java_slot_order_and_component_equality() {
        let mut slots = vec![None; SIZE];
        let mut plain = ItemStack::new("minecraft:wheat", 60);
        let mut named = ItemStack::new("minecraft:wheat", 4);
        named
            .data_mut()
            .set_other("minecraft:custom_name", Some(Tag::String("Seed".into())));
        slots[1] = Some(plain.clone());
        slots[2] = Some(named.clone());
        plain.count = 70;
        assert_eq!(insert(&mut slots, &plain, 64), 70);
        assert_eq!(slots[0].as_ref().unwrap().count, 64);
        assert_eq!(slots[1].as_ref().unwrap().count, 64);
        assert_eq!(slots[2], Some(named));
        assert_eq!(slots[3].as_ref().unwrap().count, 2);
    }

    #[test]
    fn private_codec_keeps_slot_components_and_rejects_duplicate_or_invalid_slots() {
        let mut slots = vec![None; SIZE];
        let mut item = ItemStack::new("minecraft:stick", 17);
        item.data_mut()
            .set_other("minecraft:max_stack_size", Some(Tag::Int(19)));
        slots[26] = Some(item);
        let records = Store::from([(
            [1, 70, 2],
            Chest {
                local: [1, 0, 0],
                slots: slots.clone(),
            },
        )]);
        assert_eq!(
            decode(&encode(&records).unwrap()).unwrap()[&[1, 70, 2]].slots,
            slots
        );
        let mut outside = records.clone();
        outside.get_mut(&[1, 70, 2]).unwrap().local = [0, 16, 0];
        assert!(encode(&outside).is_err());
        let mut attachment = Attachments::default();
        attachment.set(&STORAGE, records).unwrap();
        assert!(attachment.synced().is_empty());
        assert!(attachment.take_sync_changes().is_empty());
        let mut reloaded = Attachments::from_nbt(attachment.to_nbt());
        assert!(reloaded.synced().is_empty());
        assert_eq!(
            reloaded.get_existing(&STORAGE).unwrap().unwrap()[&[1, 70, 2]].slots,
            slots
        );
        let Tag::List(mut entries) = encode_items(&slots) else {
            unreachable!()
        };
        assert!(decode_slots(&Tag::List(vec![entries[0].clone(), entries[0].clone()])).is_none());
        let Tag::Compound(slot) = &mut entries[0] else {
            unreachable!()
        };
        slot.insert("Slot".into(), Tag::Byte(27));
        assert!(decode_slots(&Tag::List(entries)).is_none());
    }
}
