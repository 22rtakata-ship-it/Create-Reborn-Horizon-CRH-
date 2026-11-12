//! Test the exact lazy host-query and publication helpers used by the native
//! adapter. These callbacks are fixtures, not a running Horizon instance.
use create_core::{
    kinetics::{Axis, Kind, Node, Status},
    networks::{Networks, PublicationTracker, RegionId},
};
use std::{cell::Cell, collections::BTreeMap, sync::Arc};
const DIM: &str = "minecraft:overworld";
fn region(p: [i32; 3]) -> RegionId {
    RegionId([p[0] >> 4, p[2] >> 4])
}
fn shaft(x: i32) -> Node {
    Node {
        pos: [x, 0, 0],
        axis: Axis::X,
        kind: Kind::Shaft,
    }
}
fn source(rpm: f32) -> Node {
    Node {
        kind: Kind::Motor {
            facing: [1, 0, 0],
            rpm,
            capacity: 16.0,
        },
        ..shaft(0)
    }
}
fn host() -> BTreeMap<[i32; 3], Node> {
    [source(32.0), shaft(1), shaft(2)]
        .into_iter()
        .map(|n| (n.pos, n))
        .collect()
}
#[test]
fn ticking_node_override_preserves_power_when_its_entity_is_absent_from_host() {
    let mut service = Networks::default();
    let mut nodes = host();
    nodes.remove(&[0, 0, 0]);
    let view = service
        .query_with(DIM, region, source(32.0), 0, |p| nodes.get(&p).copied())
        .unwrap();
    assert_eq!(view.status(), Status::Running);
    assert_eq!(view.rpm([2, 0, 0]), 32.0);
    assert_eq!(view.capacity(), 512.0);
    let reads = Cell::new(0);
    let next = service
        .query_with(DIM, region, shaft(2), 0, |p| {
            reads.set(reads.get() + 1);
            nodes.get(&p).copied()
        })
        .unwrap();
    assert_eq!(next.id(), view.id());
    assert_eq!(reads.get(), 0);
}
#[test]
fn publications_reset_each_tick_without_rebuilding_topology() {
    let mut service = Networks::default();
    let mut publications = PublicationTracker::default();
    let nodes = host();
    let reads = Cell::new(0);
    for tick in 0..100 {
        publications.begin_turn(tick, DIM);
        let net = service
            .query_with(DIM, region, source(32.0), 0, |p| {
                reads.set(reads.get() + 1);
                nodes.get(&p).copied()
            })
            .unwrap()
            .shared_result();
        assert!(publications.first(&net));
        assert!(!publications.first(&net));
    }
    assert!(reads.get() > 0);
    assert_eq!(service.statistics().solves, 1);
    assert_eq!(service.statistics().cache_hits, 99);
}
#[test]
fn empty_frontier_event_invalidates_both_components_before_host_discovery() {
    let mut nodes = host();
    nodes.remove(&[1, 0, 0]);
    let mut service = Networks::default();
    let powered = service
        .query_with(DIM, region, source(32.0), 0, |p| nodes.get(&p).copied())
        .unwrap();
    let idle = service
        .query_with(DIM, region, shaft(2), 0, |p| nodes.get(&p).copied())
        .unwrap();
    assert_eq!(idle.status(), Status::Idle);
    nodes.insert([1, 0, 0], shaft(1));
    let event = service.invalidate_position(DIM, [1, 0, 0]);
    assert!(event.invalidated.contains(&powered.id()));
    assert!(event.invalidated.contains(&idle.id()));
    let merged = service
        .query_with(DIM, region, shaft(2), 0, |p| nodes.get(&p).copied())
        .unwrap();
    assert_eq!(merged.rpm([2, 0, 0]), 32.0);
    nodes.remove(&[1, 0, 0]);
    service.invalidate_position(DIM, [1, 0, 0]);
    assert_eq!(
        service
            .query_with(DIM, region, shaft(2), 0, |p| nodes.get(&p).copied())
            .unwrap()
            .status(),
        Status::Idle
    );
}
#[test]
fn published_generation_stays_stable_until_an_explicit_source_event() {
    let mut service = Networks::default();
    let mut nodes = host();
    let published = source(32.0);
    service
        .query_with(DIM, region, published, 0, |p| nodes.get(&p).copied())
        .unwrap();
    // The native source adapter must keep supplying published generation, even
    // if a raw GUI/NBT setting has changed; this tests that service contract.
    nodes.insert([0, 0, 0], source(64.0));
    let unchanged = service
        .query_with(DIM, region, published, 0, |p| nodes.get(&p).copied())
        .unwrap();
    assert_eq!(unchanged.rpm([2, 0, 0]), 32.0);
    let updated = service
        .query_with(DIM, region, source(64.0), 0, |p| nodes.get(&p).copied())
        .unwrap();
    assert_eq!(updated.rpm([2, 0, 0]), 64.0);
    assert!(!service.is_current(DIM, &unchanged));
}
#[test]
fn publication_guard_keeps_snapshots_alive_and_releases_them_at_turn_boundary() {
    let mut tracker = PublicationTracker::default();
    let net = {
        let mut service = Networks::default();
        let nodes = host();
        service
            .query_with(DIM, region, source(32.0), 0, |p| nodes.get(&p).copied())
            .unwrap()
            .shared_result()
    };
    let weak = Arc::downgrade(&net);
    tracker.begin_turn(1, DIM);
    assert!(tracker.first(&net));
    assert!(!tracker.first(&net));
    tracker.clear();
    assert!(tracker.first(&net));
    drop(net);
    assert!(weak.upgrade().is_some());
    tracker.begin_turn(2, DIM);
    assert!(weak.upgrade().is_none());
}
#[test]
fn host_discovery_respects_dimensions_region_teardown_and_invalid_halves() {
    let mut service = Networks::default();
    let nodes = host();
    let first = service
        .query_with(DIM, region, source(32.0), 0, |p| nodes.get(&p).copied())
        .unwrap();
    let other = service
        .query_with("other", region, source(-16.0), 0, |p| {
            nodes.get(&p).copied()
        })
        .unwrap();
    assert_eq!(other.rpm([2, 0, 0]), -16.0);
    service.unload_region(DIM, region([0, 0, 0]));
    assert!(!service.is_current(DIM, &first));
    assert!(service.is_current("other", &other));
    assert!(
        service
            .query_with(DIM, region, shaft(2), 1, |_| panic!("invalid half read"))
            .is_none()
    );
}

#[test]
fn a_host_solve_reads_each_candidate_once_and_never_reads_the_ticking_start() {
    let nodes = host();
    let mut service = Networks::default();
    let mut reads = BTreeMap::new();
    service
        .query_with(DIM, region, shaft(1), 0, |p| {
            *reads.entry(p).or_insert(0) += 1;
            nodes.get(&p).copied()
        })
        .unwrap();
    assert!(!reads.contains_key(&[1, 0, 0]));
    assert!(reads.values().all(|&count| count == 1));
}
