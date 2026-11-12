//! Behavioral regression scenarios for the shared event-driven service.
use create_core::{
    kinetics::{Axis, Kind, MAX_NODES, Node, Status},
    networks::{Networks, RegionId},
};
const DIM: &str = "minecraft:overworld";
const REGION: RegionId = RegionId([0, 0]);
fn shaft(x: i32) -> Node {
    Node {
        pos: [x, 0, 0],
        axis: Axis::X,
        kind: Kind::Shaft,
    }
}
fn motor(x: i32, rpm: f32) -> Node {
    Node {
        kind: Kind::Motor {
            facing: [1, 0, 0],
            rpm,
            capacity: 16.0,
        },
        ..shaft(x)
    }
}
fn line() -> Networks {
    let mut service = Networks::default();
    for node in [motor(0, 32.0), shaft(1), shaft(2), shaft(3)] {
        service.upsert(DIM, REGION, node);
    }
    service
}

#[test]
fn stable_network_is_solved_once_even_when_queried_from_other_nodes() {
    let mut service = line();
    let first = service.query(DIM, [0, 0, 0], 0).unwrap();
    for _ in 0..1000 {
        assert_eq!(service.query(DIM, [3, 0, 0], 0).unwrap().id(), first.id());
    }
    assert_eq!(service.statistics().solves, 1);
    assert_eq!(service.statistics().cache_hits, 1000);
    assert!(!service.upsert(DIM, REGION, shaft(1)).changed);
    assert!(!service.remove(DIM, [99, 0, 0]).changed);
    assert!(service.is_current(DIM, &first));
}

#[test]
fn removing_bridge_splits_and_replacing_it_merges_both_cached_halves() {
    let mut service = line();
    let first = service.query(DIM, [0, 0, 0], 0).unwrap();
    assert_eq!(service.remove(DIM, [1, 0, 0]).invalidated, vec![first.id()]);
    assert!(!service.is_current(DIM, &first));
    let powered = service.query(DIM, [0, 0, 0], 0).unwrap();
    let idle = service.query(DIM, [3, 0, 0], 0).unwrap();
    assert_eq!(idle.status(), Status::Idle);
    let event = service.upsert(DIM, REGION, shaft(1));
    assert!(event.invalidated.contains(&powered.id()));
    assert!(event.invalidated.contains(&idle.id()));
    let joined = service.query(DIM, [3, 0, 0], 0).unwrap();
    assert_eq!(joined.rpm([3, 0, 0]), 32.0);
    assert_eq!(service.query(DIM, [0, 0, 0], 0).unwrap().id(), joined.id());
}

#[test]
fn changes_do_not_invalidate_remote_networks_or_other_dimensions() {
    let mut service = line();
    for (dim, x, rpm) in [(DIM, 100, 16.0), ("other", 0, -16.0)] {
        service.upsert(dim, REGION, motor(x, rpm));
    }
    let distant = service.query(DIM, [100, 0, 0], 0).unwrap();
    let other = service.query("other", [0, 0, 0], 0).unwrap();
    service.upsert(DIM, REGION, motor(0, 64.0));
    assert!(service.is_current(DIM, &distant));
    assert!(service.is_current("other", &other));
    assert!(!service.is_current(DIM, &other));
    assert_eq!(
        service.query(DIM, [3, 0, 0], 0).unwrap().rpm([3, 0, 0]),
        64.0
    );
    assert_eq!(other.rpm([0, 0, 0]), -16.0);
}

#[test]
fn load_change_stops_and_recovers_network_without_duplicating_capacity() {
    let mut service = line();
    let consumer = Node {
        kind: Kind::Consumer {
            facing: [-1, 0, 0],
            impact: 32.0,
        },
        ..shaft(4)
    };
    service.upsert(DIM, REGION, consumer);
    let stopped = service.query(DIM, [0, 0, 0], 0).unwrap();
    assert_eq!(stopped.status(), Status::Overstressed);
    assert_eq!(stopped.capacity(), 512.0);
    assert_eq!(stopped.stress(), 1024.0);
    assert_eq!(stopped.available_stress(), 0.0);
    service.upsert(
        DIM,
        REGION,
        Node {
            kind: Kind::Consumer {
                facing: [-1, 0, 0],
                impact: 8.0,
            },
            ..consumer
        },
    );
    let active = service.query(DIM, [4, 0, 0], 0).unwrap();
    assert_eq!(active.status(), Status::Running);
    assert_eq!(active.capacity(), 512.0);
    assert_eq!(active.available_stress(), 256.0);
}

#[test]
fn open_clutch_keeps_two_independent_ports_and_closing_it_merges_them() {
    let mut service = line();
    service.upsert(
        DIM,
        REGION,
        Node {
            kind: Kind::Clutch { powered: true },
            ..shaft(1)
        },
    );
    let positive = service.query(DIM, [1, 0, 0], 0).unwrap();
    let negative = service.query(DIM, [1, 0, 0], 1).unwrap();
    assert_ne!(positive.id(), negative.id());
    assert_eq!(positive.port_rpm([1, 0, 0], [1, 0, 0]), 0.0);
    assert_eq!(negative.port_rpm([1, 0, 0], [-1, 0, 0]), 32.0);
    assert_eq!(
        service.query(DIM, [0, 0, 0], 0).unwrap().id(),
        negative.id()
    );
    service.upsert(
        DIM,
        REGION,
        Node {
            kind: Kind::Clutch { powered: false },
            ..shaft(1)
        },
    );
    assert!(!service.is_current(DIM, &positive));
    assert!(!service.is_current(DIM, &negative));
    assert!(service.query(DIM, [1, 0, 0], 1).is_none());
    assert_eq!(
        service.query(DIM, [3, 0, 0], 0).unwrap().rpm([3, 0, 0]),
        32.0
    );
}

#[test]
fn region_unload_reload_and_region_reassignment_preserve_isolation() {
    let mut service = line();
    let next = RegionId([1, 0]);
    service.upsert(DIM, next, shaft(2));
    service.upsert(DIM, next, shaft(3));
    let old = service.query(DIM, [0, 0, 0], 0).unwrap();
    assert_eq!(service.unload_region(DIM, next).invalidated, vec![old.id()]);
    assert!(service.query(DIM, [2, 0, 0], 0).is_none());
    assert_eq!(
        service.query(DIM, [1, 0, 0], 0).unwrap().rpm([1, 0, 0]),
        32.0
    );
    service.upsert(DIM, next, shaft(2));
    service.upsert(DIM, next, shaft(3));
    assert_eq!(
        service.query(DIM, [3, 0, 0], 0).unwrap().rpm([3, 0, 0]),
        32.0
    );
    service.upsert(DIM, REGION, shaft(3));
    service.unload_region(DIM, next);
    assert_eq!(
        service.query(DIM, [3, 0, 0], 0).unwrap().status(),
        Status::Idle
    );
    assert!(!service.unload_region(DIM, next).changed);
    assert!(service.unload_dimension(DIM).changed);
    assert!(service.query(DIM, [0, 0, 0], 0).is_none());
}

#[test]
fn diagonal_gear_edit_invalidates_cached_small_to_large_ratio() {
    let mut service = Networks::default();
    let small = Node {
        pos: [0, 0, 0],
        axis: Axis::Z,
        kind: Kind::Cog { large: false },
    };
    let large = Node {
        pos: [1, 1, 0],
        axis: Axis::Z,
        kind: Kind::Cog { large: true },
    };
    service.upsert(
        DIM,
        REGION,
        Node {
            pos: [0, 0, -1],
            axis: Axis::Z,
            kind: Kind::Motor {
                facing: [0, 0, 1],
                rpm: 32.0,
                capacity: 16.0,
            },
        },
    );
    service.upsert(DIM, REGION, small);
    service.upsert(DIM, REGION, large);
    let net = service.query(DIM, small.pos, 0).unwrap();
    assert_eq!(net.rpm(large.pos), -16.0);
    service.upsert(
        DIM,
        REGION,
        Node {
            axis: Axis::X,
            ..large
        },
    );
    assert!(!service.is_current(DIM, &net));
    assert_eq!(
        service.query(DIM, large.pos, 0).unwrap().status(),
        Status::Idle
    );
}

#[test]
fn truncated_network_is_not_cached_and_can_recover_after_far_end_edits() {
    let mut service = Networks::default();
    for x in 0..MAX_NODES as i32 + 3 {
        service.upsert(DIM, REGION, shaft(x));
    }
    let truncated = service.query(DIM, [0, 0, 0], 0).unwrap();
    assert_eq!(truncated.status(), Status::TooLarge);
    assert!(!service.is_current(DIM, &truncated));
    service.query(DIM, [0, 0, 0], 0).unwrap();
    assert_eq!(service.statistics().solves, 2);
    service.remove(DIM, [MAX_NODES as i32 - 1, 0, 0]);
    assert_eq!(
        service.query(DIM, [0, 0, 0], 0).unwrap().status(),
        Status::Idle
    );
}

#[test]
fn coordinate_edges_do_not_overflow_or_connect_opposite_world_edges() {
    let mut service = Networks::default();
    for x in [i32::MIN, i32::MAX] {
        service.upsert(DIM, REGION, shaft(x));
    }
    for x in [i32::MIN, i32::MAX] {
        assert_eq!(
            service.query(DIM, [x, 0, 0], 0).unwrap().status(),
            Status::Idle
        );
    }
    assert_eq!(service.statistics().solves, 2);
    assert!(service.query(DIM, [0, 0, 0], 0).is_none());
    assert!(service.query(DIM, [i32::MIN, 0, 0], 2).is_none());
}

#[test]
fn cached_results_match_uncached_solver_through_repeated_topology_and_source_edits() {
    use create_core::kinetics::solve_half;
    use std::collections::BTreeMap;
    let mut service = Networks::default();
    let mut loaded = BTreeMap::new();
    let mut sequence = 17_u64;
    for step in 0..160 {
        sequence = sequence.wrapping_mul(6364136223846793005).wrapping_add(1);
        let x = ((sequence >> 32) % 9) as i32;
        let kind = match step % 7 {
            0 => Kind::Motor {
                facing: [1, 0, 0],
                rpm: 32.0,
                capacity: 16.0,
            },
            1 => Kind::Clutch { powered: true },
            2 => Kind::Gearshift { powered: true },
            3 => Kind::Motor {
                facing: [-1, 0, 0],
                rpm: -16.0,
                capacity: 16.0,
            },
            _ => Kind::Shaft,
        };
        if step % 7 == 6 {
            loaded.remove(&[x, 0, 0]);
            service.remove(DIM, [x, 0, 0]);
        } else {
            let node = Node { kind, ..shaft(x) };
            loaded.insert(node.pos, node);
            service.upsert(DIM, REGION, node);
        }
        for node in loaded.values().copied() {
            let halves = if matches!(node.kind, Kind::Clutch { powered: true }) {
                2
            } else {
                1
            };
            for half in 0..halves {
                let reference = solve_half(node, half, |p| loaded.get(&p).copied());
                let cached = service.query(DIM, node.pos, half).unwrap();
                assert_eq!(
                    cached.status(),
                    reference.status,
                    "event {step}, {:?}",
                    node.pos
                );
                assert_eq!(cached.capacity(), reference.capacity);
                assert_eq!(cached.stress(), reference.stress);
                for direction in [[1, 0, 0], [-1, 0, 0]] {
                    assert_eq!(
                        cached.port_rpm(node.pos, direction),
                        reference.port_speed(node.pos, direction)
                    );
                }
            }
        }
    }
}

#[test]
fn dimension_reload_and_new_world_sessions_reject_stale_snapshots() {
    let mut first = line();
    let old = first.query(DIM, [0, 0, 0], 0).unwrap();
    first.unload_dimension(DIM);
    first.upsert(DIM, REGION, motor(0, 64.0));
    let reloaded = first.query(DIM, [0, 0, 0], 0).unwrap();
    assert_ne!(old.id(), reloaded.id());
    assert!(!first.is_current(DIM, &old));
    assert_eq!(reloaded.rpm([0, 0, 0]), 64.0);
    let mut second = line();
    let foreign = second.query(DIM, [0, 0, 0], 0).unwrap();
    assert_eq!(foreign.id(), old.id());
    assert!(!second.is_current(DIM, &old));
    assert!(!first.is_current(DIM, &foreign));
}
