// Copyright (c) 2026 create-horizon contributors
// SPDX-License-Identifier: MIT

use super::{Game, create_port_contraption::mirror_sync};
use create_engine::contraption::contact::{linear_top_motion, rotary_top_motion};
use horizon_client::collisions::{EntityCollisions, SceneCollisions};
use horizon_client_render::create_contraption_scene::{KEY, Record, full_cube_payload, read};
use horizon_core::geometry::Vec3;
use horizon_world::{
    attachments::Holder,
    collision_context::{CollisionContext, CollisionEntity, CollisionItem, EntityContextKind},
    entity_move::MovementRecord,
    entity_obstruction::EntityIdentity,
};

pub(super) fn sync(game: &mut Game<'_>, data: &[u8]) {
    let previous = previous_records(game, data);
    mirror_sync(game, data);
    if let Some((column, previous)) = previous {
        carry(game, column, &previous);
    }
}

fn previous_records(game: &Game<'_>, data: &[u8]) -> Option<((i32, i32), Vec<Record>)> {
    let player = game.player.as_ref()?;
    if game.remote.is_none()
        || game.riding.is_some()
        || game.sleeping.is_some()
        || player.spectator
        || player.flying
        || player.health.health() <= 0.0
        || !player.ground.on_ground
    {
        return None;
    }
    let message = horizon_protocol::attachment_sync::decode(data).ok()?;
    let Holder::Chunk([x, z]) = message.holder else {
        return None;
    };
    if message.replace
        || !message.changes.iter().any(|change| {
            change.namespace == "create".into()
                && change.name == KEY.into()
                && change.value.is_some()
        })
    {
        return None;
    }
    let column = (x, z);
    let records = read(game.world.chunks.get(&column)?).ok()?.into_owned();
    Some((column, records))
}

fn carry(game: &mut Game<'_>, column: (i32, i32), previous: &[Record]) {
    let Some(player) = &game.player else { return };
    let Some(current) = game
        .world
        .chunks
        .get(&column)
        .and_then(|chunk| read(chunk).ok())
    else {
        return;
    };
    let bounds = player.bounds();
    let feet = bounds.bottom_center().components();
    let size = bounds.size();
    let Some(movement) = current.iter().find_map(|record| {
        let before = previous.iter().find(|before| {
            before.controller == record.controller
                && before.anchor == record.anchor
                && before.axis == record.axis
                && before.blocks == record.blocks
        })?;
        let full_blocks = record
            .blocks
            .iter()
            .filter(|(_, state)| full_cube_payload(state))
            .map(|(local, _)| *local);
        let half_extents = [size.x * 0.5, size.z * 0.5];
        // The last applied transform must match this update's predecessor.
        // Snapshots and duplicate updates never apply a second displacement.
        match (before.stroke, record.stroke) {
            (None, None)
                if record.axis == 1
                    && before.angle == record.previous_angle
                    && before.angle != record.angle =>
            {
                rotary_top_motion(
                    record.anchor,
                    [before.angle, record.angle],
                    full_blocks,
                    feet,
                    half_extents,
                )
            }
            (Some(previous), Some(current))
                if previous.offset == current.previous && previous.offset != current.offset =>
            {
                linear_top_motion(
                    before.origin_at(1.0),
                    record.origin_at(1.0),
                    full_blocks,
                    feet,
                    half_extents,
                )
            }
            _ => None,
        }
    }) else {
        return;
    };
    let movement = Vec3::new(movement[0], movement[1], movement[2]);
    if movement.length_squared() < 1.0e-14 {
        return;
    }

    let entities = game.player_entity_collisions();
    let from = player.position;
    let context = CollisionContext::for_entity(
        CollisionEntity {
            identity: EntityIdentity(0),
            kind: EntityContextKind::Player,
            y: from.y,
            descending: player.shift_key_down,
            main_hand: Some(CollisionItem::EMPTY),
        },
        false,
    );
    let mut world = EntityCollisions {
        blocks: SceneCollisions {
            world: &game.world,
            binding: game.collision,
            subject: Some(player.powder_snow_subject()),
        },
        entities: &entities,
        border: player
            .border
            .as_ref()
            .map(|border| (border, [from.x, from.z])),
    };
    // Clip platform transfer against normal terrain and entity shapes while
    // retaining the rider's ground flag and own velocity.
    let Ok(allowed) = world.resolve_with_context(bounds, movement, 0.0, false, context) else {
        return;
    };
    if allowed.length_squared() == 0.0 {
        return;
    }
    if let Some(player) = &mut game.player {
        let to = from.add_vector(allowed);
        player.history.add(MovementRecord {
            from,
            to,
            original: Some(movement),
        });
        player.position = to;
    }
}
