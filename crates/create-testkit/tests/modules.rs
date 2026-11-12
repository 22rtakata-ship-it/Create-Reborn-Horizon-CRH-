use create_contraptions::contraption::windmill::{CAPACITY_PER_RPM, Config, RotationDirection};
use create_core::kinetics::{self, Axis, Kind, Node, Status};
#[test]
fn core_transmits_and_stops_after_source_removal() {
    let nodes = create_testkit::powered_shaft(32.0);
    let active = kinetics::solve(nodes[1], |p| nodes.iter().find(|n| n.pos == p).copied());
    assert_eq!(active.status, Status::Running);
    assert_eq!(active.speed(nodes[1].pos), 32.0);
    let stopped = kinetics::solve(nodes[1], |p| (p == nodes[1].pos).then_some(nodes[1]));
    assert_eq!(stopped.status, Status::Idle);
    assert_eq!(stopped.speed(nodes[1].pos), 0.0);
}
#[test]
fn custom_windmill_supplies_shared_core_without_mechanics() {
    let rpm = Config::default().rpm(16, RotationDirection::Clockwise);
    let nodes = [
        Node {
            pos: [0, 0, 0],
            axis: Axis::X,
            kind: Kind::Motor {
                facing: [1, 0, 0],
                rpm,
                capacity: CAPACITY_PER_RPM,
            },
        },
        Node {
            pos: [1, 0, 0],
            axis: Axis::X,
            kind: Kind::Shaft,
        },
    ];
    let net = kinetics::solve(nodes[1], |p| nodes.iter().find(|n| n.pos == p).copied());
    assert_eq!(net.speed([1, 0, 0]), rpm);
    assert_eq!(net.capacity, rpm.abs() * CAPACITY_PER_RPM);
}

#[test]
fn mechanics_press_pauses_on_core_power_loss_and_applies_once() {
    use create_mechanics::pressing::{Action, PressCycle};
    let nodes = create_testkit::powered_shaft(32.0);
    let net = kinetics::solve(nodes[1], |p| nodes.iter().find(|n| n.pos == p).copied());
    let rpm = net.speed(nodes[1].pos);
    let mut cycle = PressCycle::default();
    assert_eq!(cycle.tick(rpm, true), Action::Started);
    for _ in 0..5 {
        cycle.tick(rpm, true);
    }
    let suspended = cycle;
    for _ in 0..100 {
        assert_eq!(cycle.tick(0.0, true), Action::None);
    }
    assert_eq!(cycle, suspended);
    let mut applications = 0;
    let mut finished = false;
    for _ in 0..1000 {
        match cycle.tick(rpm, true) {
            Action::Press => applications += 1,
            Action::Finished => {
                finished = true;
                break;
            }
            _ => {}
        }
    }
    assert!(finished);
    assert_eq!(applications, 1);
}
