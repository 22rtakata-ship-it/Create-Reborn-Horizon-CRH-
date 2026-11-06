# Create for Horizon

Build small factories with gears, water wheels, belts and presses. Create for Horizon is an experimental Rust port of [Create](https://github.com/Creators-of-Create/Create), maintained by [Delete228](https://github.com/Delete228).

## What you can build

- Drive a machine with a hand crank, water wheel or Creative Motor. Route power through shafts, cogs, gearboxes, clutches and encased chain drives. Redstone controls the adjustable chain gearshift.
- Move items on basic belts and depots. Press sheets, mill ingredients and use fans to wash, smoke, haunt or blast items.
- Mix recipes in a Basin, heat it with a Blaze Burner or mine a block with a stationary Mechanical Drill. Engineer’s Goggles show speed and stress. Apply a copper ingot to stripped wood to make Copper Casing.

The development builds also have bearings, pistons, Gantry Carriages and Rope Pulleys. Small moving farms can share one chest and use a portable dock to receive or unload items. [Feature roadmap](docs/roadmap.md)

## Install

1. Create a Minecraft 26.3 instance in Horizon Launcher.
2. Open **Mods → Add from GitHub** and enter `https://github.com/Delete228/create-horizon`.
3. Build the native mod, then start the instance.

**0.7.0-alpha.2** is available in the [Horizon mod registry](https://playhorizon.dev/mods/create). The 0.8 development builds add the Adjustable Chain Gearshift, Copper Casing and early contraptions. The launcher builds the mod against the Horizon and Horizon Mixins revisions in the bundle. [Installation guide](docs/installation.md)

## Original models and textures

The mod works on its own with simple models and standard Minecraft textures. **For Create's original models and textures, you need the original Create 6.0.10 JAR for Minecraft 1.21.1 (NeoForge).** Put one copy in the instance's `horizon-mods/create-resources/` folder and restart Horizon. The mod reads the resources locally. [Download the supported JAR](https://modrinth.com/mod/create/version/UjX6dr61) · [Resource setup](docs/assets.md)

## Development

[Build and test](docs/development.md) · [Compatibility](docs/compatibility.md) · [Transmission review](docs/reviews/transmission.md) · [0.8 review world](docs/reviews/0.8-test-world.md) · [Contraption review](docs/reviews/0.8-contraption-world.md)

The port's code is MIT licensed. Create's code is credited to the Creators of Create. The original art and audio remain under their authors' terms. [License](LICENSE) · [Third-party notices](THIRD_PARTY_NOTICES.md)
