# Third-party work

Create reference: Creators-of-Create/Create, commit
`4148fd8d5b05c85d88947edc00fb495a5074f57d`, development version 6.0.11.
Copyright (c) The Create Team / The Creators of Create.

The following MIT code informed or was adapted into the Rust engine:

- `content/kinetics/KineticNetwork.java`: capacity and stress scale with absolute RPM.
- `content/kinetics/RotationPropagator.java`: supported coaxial and small/large cog connections.
- `content/kinetics/press/PressingBehaviour.java`: 240-unit cycle, midpoint processing,
  clamped speed interpolation and integer truncation.
- Three basic pressing recipes: iron, gold and copper ingots to sheets. The starter
  content narrows convention tags to vanilla ingredients.

Paths above are under `src/main/java/com/simibubi/create/`, except the recipes under
`src/generated/resources/data/create/recipe/pressing/`. The MIT notice is in LICENSE.
This initial implementation is incomplete and does not claim full behavioral parity.

Create's original `src/main/resources/assets/` has a separate All Rights Reserved
license. The downloaded original jar, all automated carry-over and any preview with
those assets are local files under `.local/`. They are excluded from Git. The default
content uses original provisional JSON models and refers to vanilla textures by ID;
it embeds no Mojang textures or original Create textures.

The resource bootstrap is the published Create **6.0.10** release `UjX6dr61` on
Modrinth. Its provenance and verified hashes are in `.local/import-provenance.json`.
It is explicitly separate from the 6.0.11 source reference. Replace it using
`tools/import_create.py --jar /path/to/a/built/6.0.11.jar` when available.

Horizon and Horizon Mixins are MIT. Their original checkouts are not edited or
included in the mod. The project carries a one-line compatibility patch to Horizon
Mixins allowing Rust's built-in `recursion_limit` attribute; see `patches/` and
`tools/setup_mixins.py`. It must be applied for local native builds on this pin.
