# Launcher and registry readiness

This is an **unofficial, experimental Create port**. It has not been submitted to
or approved by the Horizon registry. The mod currently targets the exact Horizon
and Mixins commits in `compatibility.json`.

## Reviewed requirements

The [Horizon registry](https://github.com/horizon-rust/horizon-mods) lists third-party
native mods as `kind = "native-mod"`. Its `native/` directory contains Horizon's own
official mods; a third-party listing does not require moving this project's code there.

- Source is public at <https://github.com/Delete228/create-horizon>.
- The project uses MIT, preserves Create's MIT attribution and links the original
  project. See `LICENSE`, `THIRD_PARTY_NOTICES.md` and the introduction to `README.md`.
- Original Create assets and Minecraft resources are excluded from Git and every
  release package. An offline importer reads a player-supplied original JAR in the
  instance and uses Horizon's resource-pack loader. Generated resources remain in
  that instance's private cache. See `ASSETS.md` for the process and licence limits;
  this design is not a claim of permission from Create's authors.
- The delivered `0.6.0-alpha.2` native archive pins full Horizon and Horizon Mixins
  commits and passed the official `cargo-horizon-mod check --kind native-mod`.
  Public content is included in this archive; no separate content archive is needed.
- Actual compiled-game validation and its limits are described in `REPORT_06.md`,
  `DEVELOPMENT_06.md` and `VALIDATION.md`; gameplay tests use private copies of
  resources and disposable test worlds.

The registry's [mod policy](https://github.com/horizon-rust/horizon-mods/blob/main/MOD-POLICY.md)
also requires the author's right to publish, a source link matching the package,
accurate descriptions and maintainer review. This checklist records our checks;
it is not a promise of acceptance by the registry.

## Current assessment (2026-11-05)

An incomplete port can be proposed as an **experimental** native mod. The policy
requires accurate scope, source/asset rights, matching public source and package,
and review of every native version; it does not require a complete Create port.
A listing is not an endorsement by Horizon or the original Create authors.

The currently delivered checkpoint contains 27 blocks, 20 standalone items and
137 recipes. Its broad completion estimate remains informal (8–10%). Complete
contraptions, trains, Ponder and much logistics are absent. Known differences,
including the measured one-tick Press/Basin startup difference, must remain
visible in the proposed description. The owner's belt/motor presentation
feedback led to the alpha.3 candidate described in `REPORT_063.md`: its X-axis
belt pose and developer artifact packaging are fixed. A separate macOS Launcher
bundle metadata bug still blocks acceptance of the ordinary Play path.

The accepted Mixins fix is now upstream [PR #25](https://github.com/horizon-rust/horizon-mixins/pull/25),
commit `e8cd0aa38cffef49cb2e088ef716baa107148c02`. Our duplicate
[PR #26](https://github.com/horizon-rust/horizon-mixins/pull/26) was closed as
superseded. The installed `alpha.2` still pins the previously validated
`a1e13326306c6bc074b11e6a4926deb551c4083e`; closing the PR does not change that
immutable commit or the installed build. Alpha.3 has now been compiled with the
accepted tool and checked as a native archive; it is not installed in the user's
instance or published as a release.
The candidate source checkpoint is
[`8b7cca1`](https://github.com/Delete228/create-horizon/commit/8b7cca1ce486e8b4297a46eabe817d3b7ed1a407).
Accepted upstream pins are a maintenance recommendation, not a stated blanket
registry prohibition on fork commits.

## Submission checklist

- [x] Public MIT source, original attribution and asset exclusions documented.
- [x] Delivered alpha.2 native archive validated; content is bundled.
- [x] Official performance build and disposable-world checks recorded with limits.
- [x] Document the latest presentation feedback and its bounded checks.
- [x] Repin/revalidate the accepted Mixins revision in the alpha.3 candidate.
- [x] Submit the generic macOS companion fix as [Horizon #9529](https://github.com/horizon-rust/horizon/pull/9529),
      with real Launcher filesystem tests and a generated-app game launch.
- [ ] Update the installed Launcher after acceptance and validate its ordinary Play
      path. The isolated app proof does not claim that UI/install check.
- [ ] Repeat the complete clean-instance GitHub installation for that exact candidate.
      The alpha.2 GitHub API attempt hit an anonymous rate limit; its bounded
      official-build fallback is recorded in `REPORT_06.md` and is not described
      as a successful clean Add-from-GitHub run.
- [ ] Publish an explicitly experimental immutable release archive, source commit,
      checksum and clear build/feature limitations.
- [ ] Prepare and validate `mods/create.toml` using the registry's current tools,
      or use the [submission form](https://playhorizon.dev/mods/submit).
- [ ] Submit for maintainer review. Every native version requires review.

No registry submission or release publication has been made. The next reviewable
candidate should complete the checks above; a stable, complete-Create listing
would overstate the current implementation.

The launcher supports **Add from GitHub** for native source and `.hnmod` releases,
and **Import a game build** for an already compiled client. Current installation
uses the normal native-build pipeline; the older imported prototype remains a
separate development build. Neither route implies registry approval.
