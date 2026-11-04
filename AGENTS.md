# Create Horizon development rules

The owner's primary requirement is observable parity with the pinned original
Create release. Read `compatibility.json`, `COMPATIBILITY.md` and the current
increment in `PLAN.md` before changing gameplay or presentation.

- Maintain all public project documentation in English. Preserve original game
  localizations and identifiers; the documentation language does not change them.
- Port behavior into idiomatic, efficient Rust. Preserve recognizable subsystem
  boundaries and document the Java source mapping where useful; a class-for-class
  translation is not required.
- Parity includes interactions, timing, quantities, components/NBT, rotation,
  entity motion/damage, sound, particles, animations and every supported GUI/HUD.
  Interface text, units, colors, spacing, wrapping, icons, scaling and transitions
  are part of the specification. An approximation is an explicitly documented
  missing feature, not accepted visual parity.
- Do not invent replacement effects or interfaces. Compare original source and
  the reference Java client; import original assets only through the private
  player-supplied JAR workflow. Never commit or distribute original assets/JARs.
- Match the exact pinned Java release. Distinguish isolated algorithm oracles,
  native integration checks and actual cross-client comparisons in reports.
  A passing numeric test does not establish presentation parity.
- Use shared schematic fixtures for equivalent Java/Horizon constructions where
  possible. Validate every block, property and required block-entity behavior
  before changing a world. Unknown/unsupported entries must be reported and
  rejected; never silently replace them with air or vanilla placeholders.
- Keep simulation, content/recipes, transport, UI/presentation, schematics and
  moving-structure integration as separate modules. Expose deliberate extension
  contracts for future Rust addons; do not promise that Java addon JARs run here.
  A future Sable physics adapter must have a clear world/transform boundary.
- Reuse Horizon's normal movement, damage, inventory, particles and networking
  paths. Do not replace missing semantics with teleporting, direct health edits,
  decorative-only mechanisms or untested client/server duplication.
- Prefer named native hooks and narrow pinned adapters. If exact behavior needs
  an upstream API extension, implement and validate that boundary explicitly.
- Preserve existing player worlds and Prism instances. Develop and compare in
  separate instances or disposable copies. Never publish account/session data.
- Ship coherent tested increments with a prepared review world and an actionable
  visual checklist. Report covered systems and known differences; percentage
  estimates are informal and must not stand in for acceptance evidence.
