# 0.7 development: source events and enclosed chain transmission

This is an **unreleased development candidate**, version `0.7.0-alpha.1`, on
`stage/parity-06`. The installed owner review instance remains `0.6.0-alpha.2`.
Do not treat this page as a completed native-build or installation report.
Public documentation is maintained in English; imported game localizations are
unchanged. No native registry proposal is being submitted in this session.

## Scope

- Add `create:encased_chain_drive`, its original 24 block states, shaft/chain
  connections, placement, shape updates, wrench rotation/removal and two source
  crafting recipes. Its canonical entity type is `create:encased_shaft`.
  Adjustable Chain Gearshift is a separate pending feature.
- Publish generated rotation from actual initialization and setting/turn/flow
  events. Distinguish those events from reading edited NBT. Keep the existing
  component solver and processing equations; do not add a Press timer offset.
- Respect effective per-stack fire resistance in the fan's destructive blasting
  decision, including explicit removal of an item's default component.

Source reference: Create `6.0.10`, Minecraft `1.21.1`, commit
`ac0c444d9828da3453ae8cc65338e8de063286fb`. Horizon and the accepted official
Mixins tool retain the revisions in `compatibility.json`.
The candidate catalog contains 28 block items, 20 standalone items, 139 recipes
and 41 exported tags. Original models/textures/locales are imported privately
from the player's supported JAR; importer cache schema is 12.

## Source findings and acceptance boundaries

The old Press/Basin report measured native impacts at 31/93 and original impacts
at 32/94. Actual original ticker-list observation now proves **both timings can
occur**: a new motor before the Press in the real ticker order starts its cycle
earlier than a new motor after it. Placement order alone is insufficient evidence.
The development bridge observes selected ticker entries without changing them.

In the original mod, `/data` changing an initialized motor's `ScrollValue` does
not invoke `ScrollValueBehaviour.setValue` or `updateGeneratedRotation`.
The ordinary value-board packet does. Native control fixtures which used `/data`
as a live speed regulator need deliberate migration; old successful runs do not
prove the new event boundary. An isolated `Speed` seed that Java did not retain
is recorded as such, not described as proof of cold-loaded network behavior.

The candidate's source publication is bounded stationary-network work, not a
complete port of `RotationPropagator`: source ownership, network rebuilding,
flicker/breakage and every loaded/non-ticking boundary remain separate work.
Clutch/Gearshift notification is handled before the next block-entity pass;
instantaneous between-tick neighbour callback parity is not yet claimed.

Create 1.21.1 uses a `FIRE_RESISTANT` unit component. The target version instead
uses a `damage_resistant` selector. The canonical `#minecraft:is_fire` tag maps
to the original full-fire flag; explicit removal overrides the prototype.
Recognized partial selectors do not grant that full-fire flag. Arbitrary custom
tags, unrecognized IDs and malformed selectors are retained without destructive
processing when no recipe applies. This conservative target policy is not a
general cross-version component data fixer. Actual recipes retain priority over
resistance, including destruction of smokeable food by blasting.

## Verification so far

| Evidence | Result | Limit |
|---|---|---|
| Workspace Rust tests | 131 passed | Portable engine/importer/schematic coverage |
| Python build/export regression | 14 passed | Development helpers, not gameplay |
| Chain state oracle | 35,412 cases passed, 12 unchanged Java methods | Authored world/property contexts; actual cached Direction/Catnip helpers |
| Chain fallback block states | All 24 rotations match the Java generator oracle | Public fallback boxes are original port geometry, not original assets |
| Actual original chain scenes | Six axis combinations, perpendicular center, removal/reconnection passed | Settled state/RPM; no transient or pixel parity claim |
| Actual original source timing | Eight scenarios passed | Explicit ticker order, NBT/setting distinction; cold-load proof remains separate |
| Fire-resistance method/type checks | Seven Rust tests, 64 component contexts, 108 original method contexts passed | Original ItemStack presence/recipe contexts are authored in this oracle |
| Actual original fan processing | Six unit-component/default/removal/recipe scenarios passed | Eventual Depot conversion and conservation; no exact first-tick or visual claim |
| New native candidate | Pending | Build and real integration must pass before handoff |

Reproduce the source/type checks:

```sh
python3 tools/cargo.py test --locked --workspace
python3 tools/check_distribution.py
python3 tools/chain_drive_oracle.py
python3 tools/fan_resistance_checks.py --horizon /path/to/pinned/horizon
```

The live reference runners require exclusive access to the prepared disposable
Java instance. `tools/chain_drive_checks.py --java` and
`tools/fan_resistance_native_checks.py --java` produce original observations;
their native modes require an explicit binary, build receipt, prepared resources,
control CLI, local JAR and reference report. They do not touch owner worlds.

## Remaining before a review build

- [ ] Build with the accepted unchanged official Mixins executable and record all input hashes.
- [ ] Compare actual native source-event/ticker-order traces without time alignment.
- [ ] Run the shared chain scenes in native Horizon and verify a real restart.
- [ ] Run native fan component/recipe scenes, including conservative unsupported selectors.
- [ ] Migrate affected live-setting regression fixtures and run the relevant machine scenarios.
- [ ] Inspect original/native chain appearance, ordinary placement and wrench interactions.
- [ ] Prepare a separate owner review world with a concrete visual checklist.
- [ ] Publish final source/build evidence; keep registry submission deferred.
