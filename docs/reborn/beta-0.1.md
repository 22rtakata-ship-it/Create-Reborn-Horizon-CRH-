# Beta 0.1 acceptance contract

The end goal is a one-to-one behavioral port of the latest stable NeoForge Create, not a reinterpretation. The audited reference is Create 6.0.10 for Minecraft 1.21.1, tag `mc1.21.1-6.0.10`, commit `ac0c444d9828da3453ae8cc65338e8de063286fb`. GitHub releases and Modrinth's NeoForge release metadata were checked on 2026-11-13. Horizon currently targets Minecraft 26.3; that version difference is not evidence of compatibility. Update the reference deliberately when stable releases change.

Beta 0.1 is an intermediate survival milestone. It must satisfy the original twenty first-playable criteria in [requirements.pt-BR.md](requirements.pt-BR.md), not only the Core smoke test. Later features remain part of the full-port goal and must be identified as incomplete in beta release notes.

## Required evidence

- Install the exact candidate package in a real Horizon instance with prepared Minecraft resources; record platform, engine revision, package digest and resource preparation version.
- Verify original block appearances with the locally imported, pinned Create resources. Fallback textures do not demonstrate visual parity. Do not redistribute the original JAR or assets.
- Obtain materials in survival, build a Water Wheel, transmit through shafts/gears and verify speed ratios, signs, source attachment, capacity, load, overload and power loss. Repeat with clutches and gearshifts and across chunk unloading/loading.
- Verify Engineer's Goggles, Speedometer and Stressometer using the reference's display rules. Plain gears do not acquire an extra numeric HUD. Compare machine stress against the actual network values.
- Run the original press, paired crushers, ore processing, fan washing, belt/funnel transfer and industrial-alloy chain. Include full inventories, interrupted power and restart without duplicate processing.
- Build a custom windmill powering the factory, a rotating platform, harvester/storage farm and piston-moved structure. Validate collision and item transfer during assembly/disassembly.
- Restart and reload the test world: machines, inventories and contraptions must survive. Exercise chunk boundaries; reject critical crashes, duplication and world corruption.
- Record actual performance and client/server behavior for the supported release configuration. Report platform and feature limitations explicitly.

Each scenario needs a reproducible setup, expected reference behavior, observed result and attached logs or capture. Unit tests, Java adapters, compilation and package validation cannot replace these observations. Do not label a release beta while required scenarios are unrun or failing.

## Source comparison checkpoint — 2026-11-13

The pinned Java comparison tools passed locally: 75 network cases, 31,752 rotation cases, 27,648 placement cases, 9,282 processing cases, 6,388 visual-formula cases, 24,409 milling cases, 17,680 fan cases plus 566,956 timer transitions, 39,970 drilling cases, and the basin recipe filter comparison. A new stress-default comparison passed for 20 block registrations. These tools extract reference algorithms and use adapters; they do not run the original NeoForge game or establish full gameplay parity.

The Goggles provider now uses the shared `Node::impact()` value instead of a partial block whitelist. Previously it omitted impact for crushers, crafters and some actuators. The new oracle compares shared numerical defaults against `AllBlocks.java` and the zero-default rule in `BlockStressValues.java`; it does not validate native block dispatch, custom server configuration or equipment rendering.

Known parity gap: the reference RotationPropagator can break blocks for invalid/excessive rotation and opposing sources; the current component solver instead stops networks for TooFast/ConflictingSources. Matching gear ratios alone does not complete source ownership and propagation parity. Reproduce these cases in isolation before changing world mutation behavior.

Status: development preview, not beta. Prepared Horizon resources are absent from this environment; real-world acceptance remains blocked. The `crh-dev-2026-11-13` native source archives contain this correction and passed the official validator. The earlier standalone Linux executable predates it; no updated standalone executable is provided by this source-package release.

Prepared-resource setup and its distinction from the original Create artwork are explained in [game-resources.md](game-resources.md).
