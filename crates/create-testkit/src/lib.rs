//! Deterministic, engine-free construction fixtures.
use create_core::kinetics::{Axis, Kind, Node};
/// Source and shaft used to validate module interoperability.
pub fn powered_shaft(rpm: f32) -> [Node; 2] {
    [
        Node {
            pos: [0, 0, 0],
            axis: Axis::X,
            kind: Kind::Motor {
                facing: [1, 0, 0],
                rpm,
                capacity: 16.0,
            },
        },
        Node {
            pos: [1, 0, 0],
            axis: Axis::X,
            kind: Kind::Shaft,
        },
    ]
}
