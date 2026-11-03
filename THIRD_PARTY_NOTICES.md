# Third-party work

Create reference: Creators-of-Create/Create, commit
`4148fd8d5b05c85d88947edc00fb495a5074f57d`, development version 6.0.11.
Copyright (c) The Create Team / The Creators of Create.

The following MIT code informed or was adapted into the Rust engine:

- `content/kinetics/KineticNetwork.java`: capacity and stress scale with absolute RPM.
- `content/kinetics/RotationPropagator.java`: supported coaxial and small/large cog connections.
- `content/kinetics/press/PressingBehaviour.java`: 240-unit cycle, midpoint processing,
  clamped speed interpolation, integer truncation and the rendered head's motion curve.
- Three basic pressing recipes: iron, gold and copper ingots to sheets. The starter
  content narrows convention tags to vanilla ingredients.

Paths above are under `src/main/java/com/simibubi/create/`, except the recipes under
`src/generated/resources/data/create/recipe/pressing/`. The MIT notice is in LICENSE.
This initial implementation is incomplete and does not claim full behavioral parity.

Create's original `src/main/resources/assets/` has a separate All Rights Reserved
license. Development inputs and port-pack carry-over stay under `.local/`.
The optional offline importer reads a JAR supplied by the player in their instance's
`horizon-mods/create-resources/`, then caches the required resources under that
instance's `.horizon-cache/create-assets/`. It does not download or execute the JAR,
modify the installed mod or grant rights to redistribute the imported resources.
The input JAR and generated caches are excluded from Git and all release archives.
Local extraction is not a guarantee of legal clearance. See `ASSETS.md`. The default
content uses original provisional JSON models and refers to vanilla textures by ID;
it embeds no Mojang textures or original Create textures.

The resource bootstrap is the published Create **6.0.10** release `UjX6dr61` on
Modrinth. Its provenance and verified hashes are in `.local/import-provenance.json`.
It is explicitly separate from the 6.0.11 source reference. Replace it using
`tools/import_create.py --jar /path/to/a/built/6.0.11.jar` when available for port-pack
development. The in-game resource importer currently accepts only the verified
6.0.10 NeoForge 1.21.1 JAR, SHA-256
`ef87fe5709f1ba1f5b8bb20a2925b5afb4669e178fd6d8bf10c167759eefe37a`.

Horizon and Horizon Mixins are MIT. Their original checkouts are not edited or
included in the mod. Builds pin the published commit
`a1e13326306c6bc074b11e6a4926deb551c4083e` of Horizon Mixins, the head of
[upstream PR #26](https://github.com/horizon-rust/horizon-mixins/pull/26), which
recognizes Rust's built-in `recursion_limit` attribute. The launcher fetches that
commit from the official repository; no tool source is patched locally. This is
an unmerged contribution, not an upstream release or approval of this port.
