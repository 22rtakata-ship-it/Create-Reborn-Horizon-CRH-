// Copyright (c) 2026 create-horizon contributors
// SPDX-License-Identifier: MIT

use std::cell::Cell;

use super::{Game, create_port_contraption::mirror_sync};
use create_engine::contraption::{
    contact::{
        ContactBox, linear_face_separation, linear_overlap_response, linear_top_motion,
        rotary_top_motion,
    },
    rotary_contact::upright_rotary_contact,
};
use horizon_client::collisions::{EntityCollisions, LookupCollisions, SceneCollisions};
use horizon_client_render::create_contraption_scene::{
    KEY, POSE_KEY, Pos, Record, dry_slab_type, full_cube_payload, read, upright_side_payload,
};
use horizon_core::geometry::{Aabb, Vec3};
use horizon_world::{
    attachments::Holder,
    collision_context::{CollisionContext, CollisionEntity, CollisionItem, EntityContextKind},
    entity_move::MovementRecord,
    entity_obstruction::EntityIdentity,
};

#[derive(Clone, Copy)]
struct ContactClip {
    excluded_controller: Option<Pos>,
}

thread_local! {
    static CONTACT_CLIP: Cell<Option<ContactClip>> = const { Cell::new(None) };
}

fn local_contact_player<'a>(
    game: &'a Game<'_>,
) -> Option<&'a horizon_client::motion::PlayerMotion> {
    game.player.as_ref().filter(|player| {
        game.remote.is_some()
            && game.riding.is_none()
            && game.sleeping.is_none()
            && !player.spectator
            && !player.flying
            && player.health.health() > 0.0
    })
}

pub(super) fn player_collision_filter(game: &Game<'_>) -> (Option<Pos>, Option<Aabb>) {
    if let Some(clip) = CONTACT_CLIP.get() {
        // The pose filter applies to the ordinary movement query. Explicit
        // contact motion still collides with every neighboring rotor.
        return (clip.excluded_controller, None);
    }
    let standing = local_contact_player(game).map(|player| {
        horizon_client::pose::Pose::Standing
            .dimensions(player.scale)
            .bounds(player.position)
    });
    (None, standing)
}

fn entity_collisions(game: &Game<'_>, excluded_controller: Option<Pos>) -> Vec<Aabb> {
    // The ordinary collision hook also supplies conservative moving geometry.
    // OBB separation must leave its own collider out, just like Java's entity
    // query. The short-lived guard affects only this collision query.
    struct Restore(Option<ContactClip>);
    impl Drop for Restore {
        fn drop(&mut self) {
            CONTACT_CLIP.set(self.0);
        }
    }
    let _restore = Restore(CONTACT_CLIP.replace(Some(ContactClip {
        excluded_controller,
    })));
    game.player_entity_collisions()
}

pub(super) fn sync(game: &mut Game<'_>, data: &[u8]) {
    let previous = previous_records(game, data);
    mirror_sync(game, data);
    if let Some((column, previous)) = previous {
        carry(game, column, &previous);
    }
}

fn previous_records(game: &Game<'_>, data: &[u8]) -> Option<((i32, i32), Vec<Record>)> {
    local_contact_player(game)?;
    let message = horizon_protocol::attachment_sync::decode(data).ok()?;
    let Holder::Chunk([x, z]) = message.holder else {
        return None;
    };
    if message.replace
        || !message.changes.iter().any(|change| {
            change.namespace == "create".into()
                && (change.name == KEY.into() || change.name == POSE_KEY.into())
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
    let mut body = ContactBox {
        min: bounds.min_position().components(),
        max: bounds.max_position().components(),
    };
    if size.y > 1.0 {
        body.max[1] -= 2.0 / 16.0;
    }
    // Preserve the existing priority of platform transfer and linear contact
    // when a nearby rotor also has a side overlap in this column.
    let mut side = None;
    let contact = current.iter().find_map(|record| {
        let before = previous
            .iter()
            .find(|before| before.same_geometry(record))?;
        let full_blocks = record
            .blocks
            .iter()
            .filter(|(_, state)| full_cube_payload(state))
            .map(|(local, _)| *local);
        let tops = || {
            record.blocks.iter().filter_map(|(local, state)| {
                let top = if full_cube_payload(state) {
                    1.0
                } else if dry_slab_type(state)? == "bottom" {
                    0.5
                } else {
                    1.0
                };
                Some((*local, top))
            })
        };
        let half_extents = [size.x * 0.5, size.z * 0.5];
        // The last applied transform must match this update's predecessor.
        // Snapshots and duplicate updates never apply a second displacement.
        match (before.stroke, record.stroke) {
            (None, None)
                if record.axis == 1
                    && before.angle == record.previous_angle
                    && before.angle != record.angle =>
            {
                if player.ground.on_ground
                    && let Some(motion) = rotary_top_motion(
                        record.anchor,
                        [before.angle, record.angle],
                        tops(),
                        feet,
                        half_extents,
                    )
                {
                    // Create resolves this rotor's shape before moving the
                    // contact point through the ordinary world colliders.
                    return Some((motion, false, Some(record.controller)));
                }
                if side.is_some() || !upright_side_payload(record) {
                    return None;
                }
                let response = upright_rotary_contact(
                    record.anchor,
                    record.angle,
                    body,
                    player.velocity.components(),
                    full_blocks,
                    max_step,
                );
                // Keep the existing path for support, lift and step-up. This
                // pass supplies only hard separation of the side of the body.
                if response.separation[1] == 0.0 && response.separation != [0.0; 3] {
                    side = Some((response.separation, true, Some(record.controller)));
                }
                None
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
                        tops(),
                        feet,
                        half_extents,
                    )
                {
                    return Some((motion, false, None));
                }
                if record.axis == 1 {
                    linear_face_separation(
                        previous_origin,
                        current_origin,
                        full_blocks,
                        body.min,
                        body.max,
                    )
                    .map(|motion| (motion, true, None))
                } else {
                    let shapes = linear_colliders(game, record, current_origin);
                    // Horizon's ordinary collision path establishes ground
                    // contact. A touching top needs no hard separation.
                    if !shapes.iter().any(|&shape| body.penetrates(shape)) {
                        return None;
                    }
                    linear_overlap_response(body, &shapes, max_step)
                        .map(|motion| (motion, true, None))
                }
            }
            _ => None,
        }
    });
    let Some((movement, separate, excluded)) = contact.or(side) else {
        return;
    };
    let movement = Vec3::new(movement[0], movement[1], movement[2]);
    if movement.length_squared() < 1.0e-14 {
        return;
    }

    let entities = entity_collisions(game, excluded);
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
    for (local, state) in record.blocks.iter() {
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
