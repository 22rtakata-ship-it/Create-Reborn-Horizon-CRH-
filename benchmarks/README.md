# Initial solver baseline

Run `python tools/reborn.py bench`. See baseline.csv for one local release-build measurement, ten solves per size. Timing includes solver traversal and network result allocation, not generation of fixture nodes. The 10000-node case deliberately asserts TooLarge; it is not a scalability success. This run does not measure event-cache hits, allocations, peak RAM, renderer, collisions or actual TPS. No performance target has been set.
