//! Solver-only wall-clock baseline; no engine, allocation or frame measurements.
use create_core::kinetics::{Axis, Kind, Node, Status, solve};
use std::{hint::black_box, time::Instant};
fn main() {
    println!("nodes,repetitions,total_microseconds,status");
    for count in [100, 1_000, 10_000] {
        let nodes: Vec<_> = (0..count)
            .map(|x| Node {
                pos: [x, 0, 0],
                axis: Axis::X,
                kind: if x == 0 {
                    Kind::Motor {
                        facing: [1, 0, 0],
                        rpm: 32.0,
                        capacity: 16.0,
                    }
                } else {
                    Kind::Shaft
                },
            })
            .collect();
        let start = Instant::now();
        let mut status = Status::Idle;
        for _ in 0..10 {
            let result = solve(nodes[0], |p| {
                (p[1] == 0 && p[2] == 0)
                    .then(|| {
                        usize::try_from(p[0])
                            .ok()
                            .and_then(|i| nodes.get(i).copied())
                    })
                    .flatten()
            });
            status = black_box(result.status);
        }
        println!("{count},10,{},{status:?}", start.elapsed().as_micros());
        assert_eq!(
            status,
            if count > 4096 {
                Status::TooLarge
            } else {
                Status::Running
            }
        );
    }
}
