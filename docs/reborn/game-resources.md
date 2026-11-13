# Prepared Horizon resources for acceptance tests

A ChatGPT desktop window does not expose the user's local filesystem to this remote workspace. The developer client here cannot launch a world without Minecraft resources prepared for the pinned Horizon engine.

Use the Horizon Launcher to install and prepare the chosen version normally. The pinned launcher resolves resources under its selected version directory at `.local/horizon-client/<game-version>/`; the directory must contain `facts.json` and `horizon-resources.json` and the files referenced by the manifest. The launcher home can be configured, so do not assume a fixed operating-system path.

If providing this prepared directory to the development environment, include its complete contents and retain relative paths. Do not include launcher accounts, tokens, player worlds or unrelated launcher folders. Confirm the installed Horizon revision and preparation version as well: resources prepared for another revision may not match this pinned client. The acceptance runner validates the manifest before launching; merely renaming a resource directory to `26.3` does not convert it.

These Minecraft resources enable game startup. They are separate from the optional local Create JAR import described in [assets.md](../assets.md), which supplies original Create artwork. Neither resource set is bundled in the public development archives. See [core-acceptance.md](../core-acceptance.md) for the runnable Core acceptance workflow, then [beta-0.1.md](beta-0.1.md) for the broader beta gates.
