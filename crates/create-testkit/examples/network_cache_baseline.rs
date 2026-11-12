//! Compare first solve, repeated cache reads and one localized source update.
use create_core::{
    kinetics::{Axis, Kind, Node, Status},
    networks::{Networks, RegionId},
};
use std::{hint::black_box, time::Instant};
fn source(rpm: f32) -> Node {
    Node {
        pos: [0, 0, 0],
        axis: Axis::X,
        kind: Kind::Motor {
            facing: [1, 0, 0],
            rpm,
            capacity: 16.0,
        },
    }
}
fn main() {
    println!(
        "nodes,register_us,first_solve_us,cached_queries,cached_queries_us,edit_and_solve_us,solves,status"
    );
    for count in [100, 1000, 10000] {
        let mut service = Networks::default();
        let start = Instant::now();
        for x in 0..count {
            service.upsert(
                "world",
                RegionId([x / 16, 0]),
                if x == 0 {
                    source(32.0)
                } else {
                    Node {
                        pos: [x, 0, 0],
                        axis: Axis::X,
                        kind: Kind::Shaft,
                    }
                },
            );
        }
        let register = start.elapsed().as_micros();
        let start = Instant::now();
        let first = service.query("world", [0, 0, 0], 0).unwrap();
        let first_solve = start.elapsed().as_micros();
        let queries = if first.status() == Status::TooLarge {
            0
        } else {
            1000
        };
        let start = Instant::now();
        for _ in 0..queries {
            black_box(service.query("world", [0, 0, 0], 0).unwrap());
        }
        let reads = start.elapsed().as_micros();
        let start = Instant::now();
        service.upsert("world", RegionId([0, 0]), source(64.0));
        let updated = service.query("world", [0, 0, 0], 0).unwrap();
        let edit = start.elapsed().as_micros();
        assert_eq!(service.statistics().solves, 2);
        assert_eq!(service.statistics().cache_hits, queries);
        if count <= 4096 {
            assert_eq!(updated.rpm([count - 1, 0, 0]), 64.0);
        }
        println!(
            "{count},{register},{first_solve},{queries},{reads},{edit},2,{:?}",
            updated.status()
        );
    }
}
