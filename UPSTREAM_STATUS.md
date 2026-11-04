# Horizon changes needed by the port

Checked 5 November 2026. The port uses a pinned, published Horizon fork.
The Launcher builds its native client automatically; users do not need to
edit engine source manually. Acceptance of one PR does not mean the entire
adapter is compatible with the latest Nightly: other boundaries must be migrated
and checked separately.

| Change | Purpose | Actual status |
|---|---|---|
| [Horizon #8872](https://github.com/horizon-rust/horizon/pull/8872) | Named mob damage and particles with their own state | Accepted and merged into main |
| [Horizon #8918](https://github.com/horizon-rust/horizon/pull/8918) | World text and atlas quads with depth testing, independent lifecycle | Updated branch pushed; open, ready for review |
| [Horizon #8924](https://github.com/horizon-rust/horizon/pull/8924) | Retained gain/pitch/position for native-mod-controlled sounds | Updated branch pushed; open, ready for review |
| [Horizon #9519](https://github.com/horizon-rust/horizon/pull/9519) | Active-world damage types, tags, exhaustion and synchronization | Corrected follow-up open, ready for review; [previous #9076](https://github.com/horizon-rust/horizon/pull/9076) closed |
| [Mixins #25](https://github.com/horizon-rust/horizon-mixins/pull/25) | Correct handling of embedded crate/tool attributes | Accepted; pinned in alpha.3 candidate |
| [Mixins #26](https://github.com/horizon-rust/horizon-mixins/pull/26) | Our version of the same fix | Closed as duplicating the already accepted fix |
| [Horizon #9529](https://github.com/horizon-rust/horizon/pull/9529), [issue #9521](https://github.com/horizon-rust/horizon/issues/9521) | Native mod metadata and post-build data beside the executable inside macOS `.app` | Fix submitted, open and ready for review; original `ensure`, data and rendering tested in a private `.app` |

These are small, generic engine boundaries, partly similar in role to mod-loader
services: other Rust mods can use them. They do not port all of Forge/NeoForge.
Create-specific recipes, behavior, interfaces and effects remain in the mod.
#9519 fixes game-data loading rather than adding a dedicated Create API.
The changes are designed to preserve normal Horizon paths, but absolute
correctness cannot be promised: review, regression and parity checks provide evidence instead.

## Changes following feedback on #9076

The maintainer identified two problems: landing read startup resources rather than
world resources, and full synchronous loading for every datapack delayed world
opening. They said the corrected version could be reopened. The author's GitHub UI
does not show Reopen for the closed PR, so a linked follow-up,
[#9519](https://github.com/horizon-rust/horizon/pull/9519), was prepared from **the same
branch**, without rewriting history. The old discussion contains a
[replacement link](https://github.com/horizon-rust/horizon/pull/9076#issuecomment-6643769160).

Branch [`data-driven-damage` at 002cf12b](https://github.com/Delete228/horizon/tree/002cf12bf264541bba7306d7f12ee2903249ecc6)
is already pushed. Landing uses the active Health catalog; eager full SyncData
is required only for a pack defining damage types. Other packs retain lazy
loading, while definitions are frozen for the world session. A regression covers removal
of all packs before the first hub connection.

At the final commit, 10 focused resource tests and 9 fresh comparisons with
Minecraft 26.3 passed: 69,531 fixtures, including 411 codec cases. Strict all-target Clippy,
formatting and public-tree/history checks passed. The earlier full self-check
is preserved separately and is not described as entirely green: it retains the unchanged
macOS installer fixture and 1-ULP item-drop failure. Testing the control function on main
is not the same as a full main run. The English PR description retains these limits.

## GitHub and publication

The connected GitHub integration recognizes account Delete228 and can read source,
discussions and CI. Attempts to create/update an upstream issue or PR return
`403 Resource not accessible by integration`. SSH push to owned repositories
works. GitHub Desktop is not required.

The working fallback is Firefox with an existing login. This was used to mark two drafts
ready for review and close the duplicate Mixins PR. After browser control recovered,
#9519, the link in #9076, macOS issue #9521 and PR #9529 were published.
The tool's earlier Mac-lock message is not treated as evidence that the user
actually locked the screen.
For the new Launcher fix, [Horizon rules](https://github.com/horizon-rust/horizon/blob/main/AGENTS.md)
require **“Claim before you start”** — first an issue and `/claim`.
[The claim was posted](https://github.com/horizon-rust/horizon/issues/9521#issuecomment-6643836877)
before production work on the Launcher fix began.

At this checkpoint, account Delete228 has exactly four open PRs in the main
Horizon repository: #8918, #8924, #9519 and #9529. All are ready for review; none are drafts.

## macOS Launcher fix

[#9529](https://github.com/horizon-rust/horizon/pull/9529) changes only
`launcher/src/game_app.rs`, commit `69e6298905d31b615e22696927c957dfb49a736c`.
Metadata is refreshed through atomic hardlink replacement independently of the executable;
`native-data` is exposed through a relative link to the version folder. Missing
companions are removed; unexpected real data is preserved with an error
and the standard fallback to the root executable.

Eight focused tests in the actual Launcher, strict all-target Clippy
and the full Launcher set passed: 524 passed, 7 pre-existing ignored. The full self-check retains
one Python failure in the unchanged installer fixture; a narrow control reproduces
it using exact main `83ad384f4` files. An all-green run of the entire main branch is not claimed.

A separate probe with declared stubs only for localization/XML called actual
unchanged `ensure`. The resulting `.app` process read real metadata and authored
post-build fixture data through actual `native_data::built`. The executable from that
`.app` then ran the game: digits 128 and the white frame appeared. Before/after images are in the PR.
This did not automate Play in the installed Launcher: updating the user installation
and a clean candidate native-build remain separate checks.

PRs #8918, #8924, #9519 and #9529 passed CI verify and launcher; fork parity needs separate
maintainer permission and is not marked as passed. We have not merged any open PR
into main Horizon ourselves.

For the mod registry, see [REGISTRY_READINESS.md](REGISTRY_READINESS.md): an accurately
described experimental package can be prepared after a complete installation run; calling
the current port a complete or stable Create implementation would be premature.
