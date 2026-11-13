//! Actual shared impact values used by simulation and the native Goggles provider.
use create_horizon_core::{
    kinetics::{Axis, Kind, Node},
    placement::Direction,
};
fn main() {
    let consumer = |impact| Kind::Consumer {
        facing: [1, 0, 0],
        impact,
    };
    let cases = [
        ("shaft", Kind::Shaft),
        ("cogwheel", Kind::Cog { large: false }),
        ("large_cogwheel", Kind::Cog { large: true }),
        ("gearbox", Kind::Gearbox),
        ("clutch", Kind::Clutch { powered: false }),
        ("gearshift", Kind::Gearshift { powered: true }),
        ("crushing_wheel", Kind::CrushingWheel),
        ("mechanical_press", Kind::Press),
        ("millstone", Kind::Millstone),
        ("mechanical_mixer", Kind::Mixer),
        ("mechanical_pump", Kind::Pump),
        ("mechanical_crafter", Kind::Crafter),
        ("deployer", Kind::LinearActuator),
        ("rope_pulley", Kind::LinearActuator),
        ("mechanical_drill", consumer(4.0)),
        ("mechanical_saw", consumer(4.0)),
        ("mechanical_bearing", consumer(4.0)),
        ("clockwork_bearing", consumer(4.0)),
        ("encased_fan", consumer(2.0)),
        (
            "gantry_carriage",
            Kind::GantryCarriage {
                facing: Direction::East,
            },
        ),
    ];
    let result: std::collections::BTreeMap<_, _> = cases
        .into_iter()
        .map(|(id, kind)| {
            (
                id,
                Node {
                    pos: [0; 3],
                    axis: Axis::X,
                    kind,
                }
                .impact(),
            )
        })
        .collect();
    println!("{}", serde_json::to_string(&result).unwrap());
}
