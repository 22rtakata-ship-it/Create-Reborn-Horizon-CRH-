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
- Native archives pin full Horizon and Horizon Mixins commits and pass the official
  `cargo-horizon-mod check --kind native-mod`. The content archive passes its own check.
- Actual compiled-game validation is described in `VALIDATION.md`; gameplay tests
  use private copies of resources and disposable test worlds.

The registry's [mod policy](https://github.com/horizon-rust/horizon-mods/blob/main/MOD-POLICY.md)
also requires the author's right to publish, a source link matching the package,
accurate descriptions and maintainer review. This checklist records our checks;
it is not a promise of acceptance by the registry.

## Remaining before a registry submission

1. The launcher pipeline now pins the published head of Mixins
   [PR #26](https://github.com/horizon-rust/horizon-mixins/pull/26),
   `a1e13326306c6bc074b11e6a4926deb551c4083e`, fetched from the official Git
   repository. Local tool patching is no longer part of the build. Check the
   current upstream review status, then repin and revalidate on an accepted revision before
   requesting a stable registry listing. Full Linux CI for the proposed fix
   [passed](https://github.com/Delete228/horizon-mixins/actions/runs/41644514346).
2. The native source/package carries its complete public content in `data/` and
   materializes it through the ordinary content loader when `create` is enabled.
   A separate content archive is not required for installation. Validate the
   complete clean-instance/GitHub path for each published version, not just its
   archive structure. See [LAUNCHER.md](LAUNCHER.md) and [VALIDATION.md](VALIDATION.md).
3. Publish an explicitly experimental GitHub release with the validated public
   archives, source commit, checksums and build limitations.
4. Prepare `mods/<id>.toml` with the owner, author, licence, source, description,
   version and immutable release asset/checksum; validate with the registry tools.
5. Submit for maintainer review when the launcher path is reproducible. Every
   native version requires review under the registry's current policy.

The launcher supports **Add from GitHub** for native source and `.hnmod` releases,
and **Import a game build** for an already compiled client. Current installation
uses the normal native-build pipeline; the older imported prototype remains a
separate development build. Neither route implies registry approval.
