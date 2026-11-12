# Performance

No FPS/TPS, rendering or memory claims are made. The inherited solver uses iterative BFS and a 4096-vertex bound; the native adapters and caching must be profiled before making graph changes. Benchmarks at 100/1000 are possible; 10000 currently exercises rejection rather than a successful network. No target is invented.

The baseline benchmark command measures headless network solving only. It does not measure topology mutation, allocations, world ticks, contraption collision, render cost or frametime. Add those after a native client can run.
