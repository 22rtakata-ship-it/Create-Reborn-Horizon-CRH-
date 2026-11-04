# Original resources from your JAR

The port does not distribute Create's original textures and models. At startup it
can read them from a JAR that you place in your Horizon instance yourself.
Neither Java nor NeoForge is started for import, and nothing is downloaded.

## Installation

1. Install the mod from GitHub and build it with the Launcher: [instructions](LAUNCHER.md).
2. Obtain original **Create 6.0.10 for Minecraft 1.21.1, NeoForge** from the
   [Create release on Modrinth](https://modrinth.com/mod/create/version/UjX6dr61).
3. Open the desired instance folder in Horizon Launcher. Create
   `horizon-mods/create-resources` there and place **one** JAR inside. Its filename
   is arbitrary; the extension must be `.jar`. There is no need to unpack or run it.
4. Start the instance. Required resources are extracted automatically before models load.

```text
<instance-folder>/
  horizon-native-mods/
    create.hnmod
  horizon-mods/
    create-resources/
      create-1.21.1-6.0.10.jar
```

For example, the input folder for a separate 0.5 instance on macOS looks like:

```text
~/Library/Application Support/horizon/instances/<instance-id>/horizon-mods/create-resources/
```

The supported, verified JAR has this SHA-256:

```text
ef87fe5709f1ba1f5b8bb20a2925b5afb4669e178fd6d8bf10c167759eefe37a
```

Other releases, Fabric versions, addons, sources/dev JARs and modified JARs
are not accepted. Renaming a file does not change its checksum.

## What happens automatically

Version 0.5 imports resources for 24 implemented blocks and all 39 Create tab entries,
including the Millstone, fan, drill, Propeller, Flour and Dough.
The mod follows parent model and texture references, preserves PNG animation
metadata and imports original `sounds.json` with 38 OGG sound-bank files.
All 61 Create language files, including Russian and English, are preserved unchanged.
The verified JAR supplies 288 imported files, including the original license file.
The private cache generates another 36 wheel material variants inheriting
original OBJ geometry. The total is 324 files; cache schema 7 refreshes the previous set.
Horizon reads OBJ/MTL through its standard loader; Java loaders are not run.
Other resources in the JAR do not add the rest of Create's machines to the game.
The Millstone uses original `millstone/block` and `millstone/inner`; the fan uses
`encased_fan/block` and `encased_fan/propeller`, plus the existing half shaft.
The drill adds `mechanical_drill/block` and rotating `mechanical_drill/head`.
Import is limited to standard JSON models and Horizon's supported
NeoForge OBJ/MTL format; unknown Java model loaders are rejected.

The result stays in **this instance**:

```text
.horizon-cache/create-assets/
  imports/<importer-version-and-hash>/
    import.json
    pack/
      pack.mcmeta
      CREATE-LICENSE.md
      assets/create/...
  overlay/...
```

Horizon loads this pack through its standard resource-pack loader.
The importer does not modify the original JAR, installed `.hmod`, original
Minecraft resources or worlds. Java classes, recipes and unrelated JAR files are not extracted.
Later starts check the cache and reuse prepared files. Missing or
damaged files are imported again. A new set is published only after
complete validation; concurrent imports are protected by a file lock.

Without a JAR, with an unsupported file or with multiple JARs, the game uses placeholder
resources. The reason is written to the Horizon log; on first start,
`create-resources/README.txt` appears with instructions. To disable original resources,
remove the JAR and restart the game or reload resource packs. Import runs
on both paths; an old cache does not enable it without the JAR.

## License

[Create's license](https://github.com/Creators-of-Create/Create/blob/mc1.21.1/dev/LICENSE.md)
separates MIT code from **All Rights Reserved** `src/main/resources/assets/`.
This mechanism excludes original resources from our repository and releases,
but grants no new rights to use or distribute them and does not
guarantee the port's legal acceptability. Rights to imported files
remain with their authors. Do not publish the JAR or generated cache with
our mod. Distributing the resources themselves requires separate permission.

## For developers

A normal build neither requires nor downloads the original JAR:

```sh
python3 tools/package.py
python3 tools/play.py --create-jar /absolute/path/to/create-1.21.1-6.0.10.jar
```

`--create-jar` only copies the supplied file into a separate
development instance, `.local/play-home/`. The game then imports it.
To test import without starting graphics:

```sh
python3 tools/cargo.py run --locked -p create-assets -- /absolute/path/to/instance 97 1
```

This tool uses the same importer as the game. It creates a local
resource pack, not a distributable mod archive. This mechanism replaces the former
original-resource packaging through `package.py --with-original-assets`.
