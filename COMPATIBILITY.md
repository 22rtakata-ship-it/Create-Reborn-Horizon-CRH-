# Original Create identifiers and data

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

Static casings/storage blocks and alloy/metal/wrench items retain their upstream
`create:*` IDs. Encased shafts unwrap to and drop `create:shaft`.

The three sheet items retain `create:iron_sheet`, `create:golden_sheet` and
`create:copper_sheet`. Kinetic entities retain `Speed` as a float. Depot data
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
basis; physical port ratios are preserved. Hand crank rendered motion is still an
approximation of the original independent/chasing angle. Complete that schema alongside the corresponding behaviors, retaining
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
