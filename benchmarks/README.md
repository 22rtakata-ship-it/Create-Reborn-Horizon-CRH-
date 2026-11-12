# Initial solver baseline

Run `python tools/reborn.py bench`. See baseline.csv for one local release-build measurement, ten solves per size. Timing includes solver traversal and network result allocation, not generation of fixture nodes. The 10000-node case deliberately asserts TooLarge; it is not a scalability success. This run does not measure event-cache hits, allocations, peak RAM, renderer, collisions or actual TPS. No performance target has been set.

## Event-driven service baseline

network-cache-baseline.csv records a separate later run in a restored environment. `cargo run --locked --release -p create-testkit --example network_cache_baseline` measures registration, first solve, 1000 cache reads and a source update plus solve. The example asserts one initial solve and one after the edit, with no solves during cached reads. Sizes 100 and 1000 run; 10000 is rejected and skipped for cached reads. Region labels in this synthetic example group 16 adjacent nodes; this is not an assumption about Horizon chunk dimensions.

These timings are single local wall-clock observations, not comparable FPS/TPS or allocation/memory measurements and not a controlled comparison against the older baseline from a different environment.
