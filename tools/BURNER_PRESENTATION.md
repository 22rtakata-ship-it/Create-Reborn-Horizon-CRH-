# Burner presentation checks

The reference is Create 6.0.10 at
`ac0c444d9828da3453ae8cc65338e8de063286fb`. The Java sources used here are MIT;
original models, textures and generated probes remain in the private local cache.

Two source renderers are deliberate separate specifications:

- `Visual` models `BlazeBurnerVisual`, including retained rods/flame/hat choices
  and its constructor's inert goggles choice. The native adapter currently uses
  this model.
- `RendererFrame` models the stateless `BlazeBurnerRenderer.renderShared`
  fallback. It is a test/reference boundary with no native consumer or automatic
  backend selection. Its different goggles, hat transform, materials and UV
  arithmetic must not be presented as runtime features.

Run the CPU checks from the repository root:

```sh
python3 tools/burner_visual_oracle.py
python3 tools/burner_renderer_oracle.py
python3 tools/burner_cache_checks.py --horizon ../horizon-port-runtime
```

The visual oracle exports unchanged methods from the pinned source and executes
the player's actual Catnip animation, angle, Minecraft trigonometry and random
dependencies. Separate disposal cases record instances around unchanged
`animate`, `setupFlameInstance` and `_delete`: heat transitions and visual
recreation must delete every created instance exactly once. These recording
contexts do not establish actual GPU allocation, client scheduling or images.

The renderer oracle records unchanged `renderShared`, `getBlazeModel` and `draw`
poses, model/material selection, fullbright and UV operations. It proves that
independent reference model, not which backend rendered a reference screenshot.

The cache check compiles the native adapter's unchanged cache implementations
and `reset`, plus the actual Horizon `ModRevision` declaration/`same_entity`.
Controlled identity inputs test ordinary NBT/property writes, replacement,
unload, dimension/world switches, resource reload, frozen diagnostic bounds and
the reset callback. Its dimension-query context is authored; it is not an actual
Game/chunk-loader/packet test. Callback wiring is checked separately in the same
driver. A resource reload recreates visual choices while retaining the live BE's
chasers and unsaved phase. World close and entity replacement discard them.

The opt-in `CREATE_HORIZON_TRACE_POSES` diagnostic retains only live client BE
identities. World-time expiration alone cannot bound it during `/tick freeze`.
Its snapshots describe actual render input, not the resulting pixels.

## Current unresolved image differences

The paired private twelve-case captures show darker native dynamic heads/rods
and a goggles-size mismatch in case 10. These remain defects under investigation;
passing numeric oracles do not close them. No gamma, texture, shading or backend
selection adjustment has been made without supporting evidence.

Eliminated or limited hypotheses:

- The supplied JAR's `BlazeBurnerVisual` matches the pinned source's retained
  constructor decisions. This is not an established JAR/source mismatch.
- All fourteen imported native parts resolve to shaded faces without explicit
  per-face colors. `NativeModelShade::Entity` and `Model` therefore use identical
  shade flags for these parts; changing that option cannot explain the observed
  difference.
- API9 fullbright still applies its diffuse term. The relevant native shader and
  Catnip's entity-style diffuse calculation use the same classic formula; this
  alone does not establish identical baked inputs or the actual source path.
- A historical reference log selected Flywheel indirect and then instancing.
  The registered visual requests skipping the vanilla renderer. A new narrow
  public-API backend observer compiled, but its private client restart failed
  before game initialization because the OS exposed zero active displays. One
  permitted temporary wake still reported zero; no repeated launches followed.
  The historical log is not a per-case observation of the captured renderer.
- An isolated original `BakedQuad.isShade` probe could not initialize the model
  loader outside FML access transformation. That failed setup is not accepted
  evidence about source baked faces.

Actual backend observation and new paired captures require an available display
and an explicitly coordinated private reference/native session.
