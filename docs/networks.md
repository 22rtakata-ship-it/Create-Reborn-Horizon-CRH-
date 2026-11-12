# Event-driven Core network service

`create_core::networks::Networks` is a game-independent registry and cache for one world session. It reuses the existing kinetic solver and source/stress equations. The native bundle engine exports the same module from the canonical source; the live Horizon ticker has not switched to this service yet.

## Host contract

Publish each loaded component using `upsert(dimension, region, node)`. Publish a new Node whenever orientation, redstone-controlled transmission, source RPM/capacity or consumer impact changes. Identical finite nodes in the same region are no-ops. A changed node invalidates cached components touching its connection neighborhood, including disconnected components that a new bridge can join. Repeated edits before a query trigger no intermediate solves.

`remove` handles block removal. `unload_region` removes only positions in the host-assigned region index; it does not scan every loaded component. RegionId groups are supplied by the host, with no assumed Horizon chunk size or invented chunk API. `unload_dimension` releases the complete dimension. Use a new service for a new world session. Do not feed unloaded block entities or client predictions into the authoritative registry.

`query(dimension, position, half)` lazily solves and caches the connected component. Half 0 is the ordinary vertex and the positive-axis side of a powered Clutch; half 1 is only valid for the powered Clutch's negative-axis side. Unknown positions and invalid halves return None. Use `port_rpm` for face-specific transmission, particularly Clutches and Gearboxes.

The returned immutable NetworkView exposes opaque, ephemeral NetworkId, status, RPM, port RPM, capacity, impact total and usable remaining stress. Views share the result through Arc rather than copying a graph. After events, query again or check `is_current`; a retained old view is an intentional snapshot, never a live engine reference. A view from another service is rejected even if its numeric ID coincides. Hosts should release old snapshots when finished. Views/IDs/caches must not be persisted: rebuild from loaded authoritative block data.

Mutation returns a Change with the invalidated network IDs. There is no growing event queue, tick scan, background worker, new dependency or unsafe code. Statistics report solves, cache hits and invalidations. Actual machine-functioning transitions can be derived by comparing newly queried status with the previous status; this increment does not add a separate machine callback scheduler.

## Topology and bounds

The solver and invalidator share `kinetics::connection_candidates`, the existing axial/diagonal candidate footprint. Candidate coordinates use checked arithmetic so world-edge queries cannot wrap or panic. Source balance and ratios are unchanged; traversal remains iterative. Hitting MAX_NODES stops candidate expansion, returns TooLarge and supplies no usable RPM. This deliberately tightens the previous nested-loop early exit rather than continuing other neighbor slices after the limit.

TooLarge results are not cached: a partial traversal cannot see an edit beyond its truncated frontier. The 4096-vertex bound remains. This service does not promise successful 10000-node networks.

## Verification and integration boundary

See `crates/create-testkit/tests/network_events.rs` for topology split/merge, remote/dimension isolation, source changes, overload recovery, clutch halves, diagonal cogs, region unload/reload/reassignment, stale-session snapshots, coordinate edges, traversal bounds and repeated edits compared against an uncached solver. A 1000-query assertion checks that an unchanged network is solved once.

The existing native kinetic_runtime.rs still keys its own cache by game_time and retains source initialization/publication ordering. Replacing it requires mapping the pinned engine's real block/chunk callbacks, feeding only published generation values (not raw NBT settings), handling temporarily removed ticking block entities, preserving synchronous source propagation and running native regression fixtures. No new Horizon hook or engine API is introduced here. This checkpoint has no in-game performance or gameplay claim.
