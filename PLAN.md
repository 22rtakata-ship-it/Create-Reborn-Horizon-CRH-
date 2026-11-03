# Port plan and review checkpoints

Agreed with the project owner on 2026-11-03. The goal is a complete, unofficial
Rust implementation of Create for Horizon, delivered in reviewable increments.

## Working agreement

Each increment has a bounded feature list, automated checks, a reproducible
source commit and build, known limitations and a short visual checklist. Stop
after delivering it and wait for the owner's feedback before starting the next
increment. Automated verification is the developer's responsibility; the owner
tests appearance and interactions. Record every observation as a defect, an
unimplemented feature or a question; retain observations about later stages.

## Milestones

1. **Installation and builds (awaiting visual review).** Build through the official launcher CLI
   without local tool patches; carry code and public content in one native
   bundle; test a clean instance and GitHub source installation. Preserve normal
   Horizon instances. Deliver a launchable review instance and exact instructions.
2. **Appearance of the initial set.** Original models and textures from the
   player-supplied JAR; inventory, held, dropped and placed items; all supported
   orientations, lighting and animation. Review a visual test gallery.
3. **Behavior of the initial set.** Pin matching Create source/assets/reference
   versions. Compare Java and Rust kinetics, connection rules, direction, stress,
   pressing and depot interactions. Review a working small factory.
4. **Complete initial release.** Saves, automation, client/server synchronization,
   performance, recovery and repeated clean installation. Publish an accurately
   scoped experimental release after the review checkpoint.
5. **Expand by subsystem.** Mechanical transmissions and belts; processing;
   logistics; fluids; moving contraptions; trains. Split each subsystem into
   bounded increments with the same tests and visual review gates.
6. **Tutorials and completeness.** Ponder, remaining interfaces/effects,
   compatibility and an inventory of every original feature and known difference.

## Architecture and verification

Keep portable behavior in Rust crates and Horizon integration in a small adapter.
Use documented named hooks where possible; raw source mixins pin the Horizon
commit and must be revalidated on updates. Prefer internal workspace libraries;
extract reusable public libraries when their interfaces are established.

Use unit and boundary tests, Java-reference comparisons, stateful world scenarios,
save/reload checks, render checks, and actual launcher installation tests.
Test counts alone are not a completeness claim. CI records exact versions.

## Distribution

The target is a GitHub URL or a reviewed Horizon registry entry followed by the
launcher's normal native build. Original Create assets are not redistributed;
the player supplies the supported JAR locally. Registry acceptance is the
maintainers' decision and does not make this an official Horizon/Create project.

## Current review checkpoint

The installation increment is ready for owner review. `Create Native` uses the
launcher's `performance` build `native-fc02103-1ba8788c96b6`; the independent
`CreateInstallCheck` world contains the initial factory and all nine prototype
items. Automated installation, world and renderer checks passed. The launcher
shows the GitHub mod and its build as **Up to date**. Next: collect the owner's
visual observations, then agree the bounded appearance increment before coding it.
