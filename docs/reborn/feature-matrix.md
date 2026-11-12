# Desired scope versus existing implementation

Evidence levels: **local unit** means the inherited Rust tests were executed in this environment; **source** means inspected adapter/data paths; **author** means upstream gameplay report only. No feature below has been newly verified in a running Horizon client here. Detailed source and hook inventories accompany this matrix.

| Desired system | Existing evidence | Reuse decision / remaining acceptance |
|---|---|---|
| RPM, stress, ratios, shafts/cogs/gearbox | local unit; kinetics, placement, chain_drive | Reuse; Core now has dimension/region lifecycle and a persistent event-only cache tested headlessly; native calls now integrated/type-check gate tracked in native-networks.md; real-game timing and 10000-node support remain and full split/merge/unload scenarios remain |
| Water wheels, crank, motor | local unit; waterwheel, controls; source generators | Reuse values and flow rules; real interaction, survival and visual tests required |
| Clutch, gearshift, chain drive | local unit; kinetics/chain_drive | Reuse; redstone propagation in actual chunks remains unverified here |
| Speed controller, gauges, tools | source native adapters, author fixtures | Preserve; check live RPM/stress/UI and source behavior |
| Items, recipes, zinc generation | source content/data and native registration | Preserve; new survival world acceptance required |
| Press and Millstone | local unit; pressing/depot/milling | Reuse; machine cycles and inventory boundary tests inherited, native transfers not reproduced |
| Crushing Wheels | local unit crushing; source native controller/recipes | Reuse bounded implementation; non-downward arrangements, entity damage and full receiver matrix remain |
| Fan washing/blasting/smoking/haunting | local unit fan/transport; source tags and recipes | Reuse; real fluid/obstacle/entity interactions and ore-line output conservation required |
| Belts, depots, funnels/chutes | local unit transport/chute/depot; source adapters | Reuse; slope/passenger coverage and automation/restart tests required |
| Basin/Mixer/Burner | local unit basin/burner; source recipe/fluid integration | Reuse; native heat, recipe reload and mid-cycle saves need actual client |
| Drill/Saw/Harvester/Deployer/Crafter | local unit algorithms; source action adapters | Reuse with exact limits; permissions, loot, full inventory and cross-chunk behavior remain gates |
| Assembly, glue, chassis, bearing | local unit contraption suite; source native capture/restore | Reuse; author documents single-column capture/restore, not arbitrary chunk-safe assembly |
| Custom windmill | local unit sails/windmill; new Core interoperability fixture | Reuse shared Core source contract; real factory and save/reload required |
| Piston, pulley, gantry | local unit geometry/drivers; source native adapters | Reuse; collision, unloading and native persistence acceptance required |
| Mobile storage and actors | local unit storage/actor policy; author gameplay | Preserve; no local real-world corruption/duplication certification |
| Persistence | local unit snapshot helpers, source codecs, author restart fixtures | Do not claim all machines versioned/migratable; audit every saved format |
| Multiplayer | source LAN/sync hooks | Unsupported until real server/client tests pass |
| Rendering | source draw/visual adapters, author captures | Preserve built-in resources/importer; no FPS or presentation parity claim |
| Packaging | existing source bundle, manifest, official Mixins path | Reuse single bundle; exact local validation in validation.md |
| Fluids: pipes, pump, tank, drain | local unit fluid_transport; native fluid codecs/adapters | Preserve current bounded routing/storage; generalized fluid API, advanced devices and native save/reload acceptance remain |
| Tunnels, arm, hose pulley, vault, spout, elevators | No completed local behavior validation | Audit and implement incrementally; no support inferred from names or catalog entries |
| Sequenced Gearshift / redstone control | local unit sequenced; native control integration | Preserve; exact motion sequences and world redstone acceptance remain |
| Trains, Ponder, advanced expansions | absent or partial exploratory work | Deferred; do not infer gameplay support from source names |

Audit scope: the complete tree was inventoried; key module dependencies, graph algorithm, manifests, licenses and build tooling were inspected. This is not a completed behavioral review of every source line or every authored test fixture. Existing author-only evidence must be rerun before a playable release.
