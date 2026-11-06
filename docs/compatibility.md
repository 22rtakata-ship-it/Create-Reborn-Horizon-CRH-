# Original Create identifiers and data

The 0.8.0-alpha.13 development checkpoint adds Chute input into a moving
chest through a portable item dock and ordinary bone meal in a stationary
Deployer. Moving tool visuals use the driver's reported stall state. Capture
and restoration currently share one chunk column. See the supported drivers,
payloads, actors and verification in the [contraption guide](contraptions.md).

The sections marked 0.3–0.5 below retain the released historical boundaries.
The installed review checkpoint is **0.6.0-alpha.2**: see [REPORT_06.md](https://github.com/Delete228/create-horizon/blob/eb8b96e8ea1db698b79bb61fd1caf6c963f31652/REPORT_06.md)
and [DEVELOPMENT_06.md](https://github.com/Delete228/create-horizon/blob/eb8b96e8ea1db698b79bb61fd1caf6c963f31652/DEVELOPMENT_06.md) for scoped integration evidence and
remaining differences. It adds limited schematic placement, bare-belt transport,
Basin/Mixer/fluid/heat processing and Press compacting, not complete Create parity.

For the new Press adapter, `Mode` retains WORLD=0, BELT=1 and BASIN=2.
`ParticleItems` is consumed once by client-update construction and is not saved;
empty later updates clear the client's retained field. The active recipe object
is transient and retained through datapack reload, not encoded as a new recipe
field in the save. Canonical input/output inventories retain unknown sibling
data; unsupported predicates/structures fail explicitly. The measured one-tick
kinetic startup difference is historical; the bounded 0.7 source-event/ticker-order
comparison supersedes it without adding a timer offset.

Version 0.7 adds `create:encased_chain_drive`
with entity type `create:encased_shaft` and the original 24 combinations of `axis`,
`axis_along_first` and `part`. The isolated original default is axis Y,
`axis_along_first=true`, `part=none`. Two source crafting recipes and the complete
source loot data are preserved. Ordinary interactions, six settled chain scenes
and real native restarts were checked in the alpha.1 build. Full propagation and
presentation parity remain open. [0.7 report](reviews/0.7-report.md)

The target's per-stack `minecraft:damage_resistant` canonical
`{types:"#minecraft:is_fire"}` maps to original 1.21.1 `FIRE_RESISTANT` presence.
Explicit removal overrides the default component. Recognized partial selectors
do not grant full-fire resistance; unknown/malformed selectors are conservatively
retained when no recipe applies. This is not a general component data fixer.
Actual recipes retain priority. The 0.5 default-only description below is historical.

The 0.7 generator adapter distinguishes source initialization, actual GUI/crank/
wrench/flow callbacks and raw NBT reads. An initialized motor's `/data` edit is
not an ordinary live settings action in original Create. Bounded Press/Basin
traces match 1,488 ordinary tick samples without time alignment; authentic
cold-loaded Java networks, source ownership and flicker/breakage remain pending.

The behavior reference is Create **6.0.10**, Minecraft **1.21.1**, source commit
`ac0c444d9828da3453ae8cc65338e8de063286fb`. The target is Horizon's Minecraft
**26.3** implementation. Cross-version Java world interchange is outside the
current requirement. Preserve upstream identifiers and data conventions when
implementing each feature, and test save/reload inside Horizon.

| Block / item ID | Block entity type ID | Implemented state / data |
| --- | --- | --- |
| `create:shaft` | `create:simple_kinetic` | `axis`, `waterlogged` (default false) |
| `create:cogwheel` | `create:simple_kinetic` | `axis`, `waterlogged` (default false) |
| `create:large_cogwheel` | `create:simple_kinetic` | `axis`, `waterlogged` (default false) |
| `create:creative_motor` | `create:motor` | `facing`; `ScrollValue` int, default 16 |
| `create:mechanical_press` | `create:mechanical_press` | horizontal `facing`; `Running`, `Finished` bytes; `Ticks`, `Mode` ints (WORLD=0, BELT=1) |
| `create:depot` | `create:depot` | `waterlogged` (default false), no facing/open state; `HeldItem`, `OutputBuffer` |
| `create:gearbox` | `create:gearbox` | `axis` |
| `create:clutch`, `create:gearshift` | corresponding block ID | `axis`, `powered` |
| `create:hand_crank` | `create:hand_crank` | `facing`, `waterlogged`; `InUse` int, `Backwards` byte |
| `create:andesite_encased_shaft`, `create:brass_encased_shaft` | `create:encased_shaft` | `axis` |
| `create:encased_chain_drive` | `create:encased_shaft` | `axis`, `axis_along_first`, `part` (`none`, `start`, `middle`, `end`) |
| `create:adjustable_chain_gearshift` | `create:adjustable_chain_gearshift` | Same chain properties and `powered`. Integer `Signal` stores the analog redstone strength. |
| `create:millstone` | `create:millstone` | no block-state properties; `Timer` int, `InputInventory`, `OutputInventory` |
| `create:encased_fan` | `create:encased_fan` | six-way `facing`, original default `down`; common kinetic `Speed` |
| `create:mechanical_drill` | `create:drill` | six-way `facing`, `waterlogged`; defaults `down`, false; `Progress`, `NextTick`, `Breaking` |

Static casings/storage blocks and alloy/metal/wrench items retain their upstream
`create:*` IDs. Encased shafts unwrap to and drop `create:shaft`.

The four sheet items retain `create:iron_sheet`, `create:golden_sheet` and
`create:copper_sheet` and `create:brass_sheet`. Kinetic entities retain `Speed` as a float. Depot data
uses `HeldItem.Item`, `Pos`, `PrevPos`, `Offset`, `PrevOffset`, `InSegment`,
`Angle`, `InDirection`; its eight output slots use `OutputBuffer.Size` and
`OutputBuffer.Items`. The creative tab uses upstream `itemGroup.create.base`.
All original locale files are imported unchanged from the player's supported
JAR; they are not included in the public repository.

## Present limits

This is an identifier/data convention baseline, not a completed Java serializer.
Horizon also saves its generic `Items` container field. Item stacks use Horizon's
26.3 item codec, not a separately implemented 1.21.1 data fixer. Depot's transport
position defaults are placeholders; belt movement is not implemented. Hopper insertion and extraction of the held
slot and output buffer follow the implemented ordinary-depot defaults. Reading a Java depot's `HeldItem` into
Horizon slots is not implemented. Waterlogged state names are declared, but
fluid placement/ticks still need their own increment.

Network serialization (`Source`, `Network`) is not fully represented yet. The prototype also stores diagnostic `Capacity`, `Stress`,
`NetworkStatus` and press bookkeeping `Applied`/`LastTick`; these are port-specific
extra fields. The `create_horizon.Ports` compound stores diagnostic physical port speeds.
Gearbox `Speed` uses a fixed reference port rather than Create's source-facing
basis; physical port ratios are preserved. Hand crank rendered motion now uses the original independent/chasing angle. Complete that schema alongside the corresponding behaviors, retaining
unknown loaded fields and documenting migrations. Existing 0.1 block entity IDs
remain accepted at runtime; newly placed blocks use the canonical type IDs above.

## Evidence

`tools/placement_oracle.py` compares Rust decisions to AST-extracted original
Java methods for 27,648 contexts. `tools/smoke.py` tests actual player placement,
depot exchange without a menu, item counts/components and save/reload, as well as
the small powered press chain. These tests do not establish full Create parity.


## 0.3 behavior boundary

The ordinary depot uses default `bulkPressing=false`, nine internal slots, direct
top exchange and a dynamic automation insertion predicate. Wire update tags carry
an explicit empty `HeldItem` compound when clearing the client; saved empty depots
omit it. This accounts for Horizon's merge semantics without leaving a stale item.
Motor updates use a bounded custom channel with server validation of packet size,
range, player, distance and target block; they do not implement Create's packet format.

The graph covers loaded components up to 4096 nodes, four-port gearboxes and split
clutch halves. Compatible generators choose the faster driver. Inconsistent cycles,
opposing sources and excessive speeds conservatively stop the component; original
breakage/network rebuilding is incomplete. Server configs are currently pinned to
the relevant Create defaults. World item scan timing/scatter, particles, random
sound variation, water and some collision/interaction shapes need later parity work.

## 0.4 data and behavior boundary

`create:speedometer` and `create:stressometer` retain `facing`, `axis_along_first`,
`Value` float and `Color` int. `create:vertical_gearbox` is an item alias for the
original `create:gearbox` block; normal/vertical recipes and axis-sensitive drops
use upstream IDs. `create:goggles` uses the head equipment slot and the original
3D head model through Horizon's item-display selection.

`create:water_wheel` retains six-way `facing`; `create:large_water_wheel` has `axis`
and `extension`. Both save `FlowScore` int and `Material.Name`, default spruce.
The invisible structural block uses the canonical `create:water_wheel_structure`
ID and `facing`. The port currently gives structural blocks a lightweight ticking
block entity to clean up missing masters; Java does not. The master also stores
`create_horizon.StructureReady` as a port-specific initialization marker.

World-axis angular transforms and crank motion have direct source/pose checks.
Gauge interpolation and comparator formulas are implemented with default config.
Native HUD layout, connection hints and rotation indicators remain provisional;
complete Java UI/Ponder/particle-provider parity is a later increment. Fluid
currents come from Horizon's vanilla sampler; Create's extra water-spread restriction
above wheel axes is not yet ported. Collision/selection shapes still include
conservative template approximations. This release adds generators, not moving
contraptions or fluid-handling machinery.

## 0.5 data and behavior boundary

The public catalog contains 24 block items and 15 standalone items: 39 creative-tab
entries, plus the internal water-wheel structural block. New item IDs are
`create:propeller`, `create:wheat_flour` and `create:dough`. The exported set has
123 source recipes and 35 tags. Processing includes 47 `create:milling`, ten
`create:splashing` and 19 `create:haunting` recipes with implemented Create or
vanilla inputs/results. Cooking uses Minecraft's real smelting, smoking and
blasting lookup; the exported dough-to-bread recipes remain
`minecraft:smelting` and `minecraft:smoking`, not invented Create recipe types.

The millstone has a fixed Y rotation axis, a shaft connection only underneath,
and the source small-cog connection behavior. Its source collision consists of a
16×6×16 base and a 12×10×12 centered upper part. One input slot and nine output
slots retain the original inventory names; Horizon also stores its generic
container representation and `create_horizon:millstone` bridge snapshot. Empty
hand collection takes output first, or input if no output exists. Recipe-filtered
automation, landed items, per-item probability rolls and timer handling are
implemented; native integration and save/reload coverage are recorded separately
in `VALIDATION.md`.

The fan's shaft is opposite `facing`; both directions and all six orientations
are represented. Airflow range, push/pull choice, processing segments, partial
obstacle sampling, propeller speed and stack-dependent processing timers are
source-derived. The implemented targets are dropped item entities and depot-held
stacks. Water washes, lit ordinary fire/campfires smoke, soul fire/campfires haunt,
and lava blasts. Blasting can destroy items, including smokeable food, according
to the source policy; it is not a general-purpose way to cook food.
The current blasting adapter reads the item's default fire resistance. Per-stack
fire/damage-resistance component overrides are not yet used for that decision;
the native destruction tests cover ordinary item definitions.

`create:fan_transparent` and block/fluid `fan_processing_catalysts/*` retain the
supported source entries; unavailable Create blocks are omitted. Until the API
exposes fluid-tag queries, the adapter recognizes vanilla water/lava IDs and
waterlogged blocks explicitly. This is not arbitrary mod-fluid compatibility.
Recipe caches refresh when server data loads or reloads.

Depot-held progress uses `HeldItem.FanProcessingType` and
`HeldItem.FanProcessingTime`. Dropped-item progress now uses the original
`NeoForgeData.CreateData.Processing.Type` and `Time` entity NBT fields, retained
through Horizon's existing persistent unknown-field storage. Native save/reload
tests verify the exact remaining timer, preserved unknown sibling keys and
one-time conversion. Item stack components are not used to
disguise entity state, and no additional raw hook is needed. Airflow on players
and mobs is not implemented. The full native 0.5 regression, including Metal,
audio and restart, has passed. Fan particles, soundscape and exhaustive
collision/media edge coverage remain separate work; no complete fan parity
is claimed.

The stationary mechanical drill uses the source rear shaft, six orientations and
12-pixel directional casing collision. Its front target, progress and retry timing
follow `DrillBlockEntity`/`BlockBreakingKineticBlockEntity`; ordinary block loot is
used rather than creative-mode drops. Its new crafting recipe uses the existing
alloy/casing/iron ingredients. Contraption movement, entity damage and automatic
target placement are outside this increment.
The native destruction path also leaves Create's cobblestone-generator shortcut,
ice-specific melting and custom zero-motion drop spawning for a later increment.

## Behavioral parity strategy

Rust organization follows the native runtime and may combine several Java
classes into cached data and small functions. Acceptance depends on observable
behavior, not a class-for-class translation. The existing AST-based oracles
compare selected unchanged Java method bodies against Rust with restricted
development contexts; they do not run the full original mod.

A future end-to-end comparison will build equivalent schematic test fixtures in
original Create and Horizon, apply the same inputs/actions, and compare output
counts/components, timing, rotation, stopping conditions and saved state. That
cross-runtime fixture runner is planned, not implemented in 0.5. It does not
change the explicit exclusion of cross-version world interchange or imply a
working schematic-import feature.
