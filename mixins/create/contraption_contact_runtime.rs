// Copyright (c) 2026 create-horizon contributors
// SPDX-License-Identifier: MIT

use super::{Game, create_port_contraption::mirror_sync};
use create_engine::contraption::contact::{
    ContactBox, linear_face_separation, linear_overlap_response, linear_top_motion,
    rotary_top_motion,
};
use horizon_client::collisions::{EntityCollisions, LookupCollisions, SceneCollisions};
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
    let max_step = game
        .player
        .as_mut()
        .and_then(|player| player.attribute("minecraft:step_height").ok())
        .map_or(0.6, |step| step as f32);
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
    let Some((movement, separate)) = current.iter().find_map(|record| {
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
                if player.ground.on_ground
                    && record.axis == 1
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
                .map(|motion| (motion, false))
            }
            (Some(previous), Some(current))
                if previous.offset == current.previous && previous.offset != current.offset =>
            {
                let previous_origin = before.origin_at(1.0);
                let current_origin = record.origin_at(1.0);
                if player.ground.on_ground
                    && let Some(motion) = linear_top_motion(
                        previous_origin,
                        current_origin,
                        full_blocks.clone(),
                        feet,
                        half_extents,
                    )
                {
                    return Some((motion, false));
                }
                let low = bounds.min_position().components();
                let mut high = bounds.max_position().components();
                if size.y > 1.0 {
                    high[1] -= 2.0 / 16.0;
                }
                if record.axis == 1 {
                    linear_face_separation(previous_origin, current_origin, full_blocks, low, high)
                        .map(|motion| (motion, true))
                } else {
                    let body = ContactBox {
                        min: low,
                        max: high,
                    };
                    let shapes = linear_colliders(game, record, current_origin);
                    // Horizon's ordinary collision path establishes ground
                    // contact. A touching top needs no hard separation.
                    if !shapes.iter().any(|&shape| body.penetrates(shape)) {
                        return None;
                    }
                    linear_overlap_response(body, &shapes, max_step).map(|motion| (motion, true))
                }
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
        if separate {
            let response = movement.components();
            let mut velocity = player.velocity.components();
            for axis in [0, 2] {
                if response[axis].abs() > 1.0 / 128.0 && velocity[axis] * response[axis] < 0.0 {
                    velocity[axis] = 0.0;
                }
            }
            player.velocity = Vec3::new(velocity[0], velocity[1], velocity[2]);
        }
        player.position = to;
    }
}

fn linear_colliders(game: &Game<'_>, record: &Record, origin: [f64; 3]) -> Vec<ContactBox> {
    let mut lookup = LookupCollisions {
        block: &|_, _, _| game.world.air,
        binding: game.collision,
    };
    let mut boxes = Vec::new();
    for (local, state) in &record.blocks {
        let at: [f64; 3] = std::array::from_fn(|i| origin[i] + f64::from(local[i]));
        if full_cube_payload(state) {
            boxes.push(ContactBox {
                min: at,
                max: at.map(|value| value + 1.0),
            });
            continue;
        }
        let Some(id) = game.pack.assets.registry.id(state) else {
            continue;
        };
        let Ok(shape) = game.collision.collision_shape(&mut lookup, id, *local) else {
            continue;
        };
        for bounds in shape.boxes() {
            let min = bounds.min_position().components();
            let max = bounds.max_position().components();
            boxes.push(ContactBox {
                min: std::array::from_fn(|i| at[i] + min[i]),
                max: std::array::from_fn(|i| at[i] + max[i]),
            });
        }
    }
    boxes
}
