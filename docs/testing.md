# Testing

Run `python tools/reborn.py verify` for Rust formatting, all workspace tests, no-default-feature isolation, Clippy with warnings denied and the inherited source distribution contract. Run `python -m unittest discover -s tools -p 'test_*.py'` for the retained Python build and recipe tooling regressions.

The root facade deliberately has no separate copy of unit tests: upstream tests execute in their owning split crates. Integration tests are under crates/create-testkit/tests/modules.rs. They verify source removal, shared windmill energy and a Press cycle paused by power loss and resumed without applying twice. They use pure simulation and do not test real inventories, NBT, players, chunks or packets.

The native bundle engine can also be tested independently: `cargo test --offline --manifest-path mixins/create/engine/Cargo.toml`. Its lockfile is generated locally because the inherited packaging excludes lockfiles. This validates the self-contained native simulation facade, not the injected Horizon client.

Do not replace the original Java/Horizon gameplay fixtures with mocks. Those tools require isolated reference instances and often private JAR inputs. The absence of those inputs is recorded in reborn/validation.md. No client/server multiplayer test has been run.

Event-driven Core regressions live in crates/create-testkit/tests/network_events.rs. They include an uncached-solver comparison across 160 edits and explicit work counters for 1000 repeated queries. Region fixtures are host-defined groups, not real Horizon chunk tests.
