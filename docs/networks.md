# Event-driven Core network service

`create_core::networks::Networks` is a game-independent registry and cache for one world session. It reuses the existing kinetic solver and source/stress equations. The native bundle engine exports the same module from the canonical source; the native kinetic_runtime.rs now resolves its networks through this service. Source initialization and publication rules remain in that adapter.

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

The native kinetic_runtime.rs now uses query_with for lazy authoritative discovery and shares solver snapshots with existing NBT/rendering code. The query start overrides the engine read at its own position, preserving the temporarily removed ticking block entity. Other sources still pass through the existing published_node/generation lifetime logic; raw settings are not newly promoted into generated rotation.

PublicationTracker resets its duplicate-publication guard on a tick or dimension change without discarding topology. It retains each published Arc until reset to prevent allocator address reuse from suppressing publication. Block-state notifications call invalidate_position, which also handles previously empty frontiers. Source/transmission/belt/bulk-operation invalidation paths remain conservative full resets. Both server chunk load and unload trigger a full reset, because a previously empty frontier can gain a connection and unload callbacks can still see departing block entities. No unloaded-world read is performed in these callbacks. Localizing chunk invalidation requires a frontier-dependency index and is future work.

The host contract is now implemented against the fixed Horizon revision, but live source timing, real chunk lifecycle, rendering and save/restart still require actual-client regression fixtures. See reborn/native-networks.md for this increment's execution evidence.

## Lazy host queries and compatibility

query_with accepts a loaded authoritative read callback and a host region mapping. Only a cache miss visits candidates; each candidate is read at most once in that solve. The start is never fetched from the callback. Existing consumers may use shared_result() to obtain an Arc of the legacy Network for NBT/presentation interfaces; the service retains its immutable cached copy. This is a compatibility bridge, not a new save format.
