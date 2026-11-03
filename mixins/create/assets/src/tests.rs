// SPDX-License-Identifier: MIT
use super::*;
use std::io::Write;
use zip::{ZipWriter, write::SimpleFileOptions};

fn fixture(change: impl FnOnce(&mut BTreeMap<String, Vec<u8>>)) -> Vec<u8> {
    let mut files = BTreeMap::new();
    let mut json_file = |name: String, value: Value| {
        files.insert(name, serde_json::to_vec(&value).unwrap());
    };
    for block in BLOCKS {
        json_file(
            format!("assets/create/blockstates/{block}.json"),
            json!({"variants": {"": {"model": "create:block/shared"}}}),
        );
    }
    for item in BLOCKS.iter().chain(ITEMS) {
        json_file(
            format!("assets/create/models/item/{item}.json"),
            json!({"parent": "create:block/shared"}),
        );
    }
    for part in PARTS {
        json_file(
            format!("assets/create/models/block/{part}.json"),
            json!({"parent": "create:block/shared"}),
        );
    }
    json_file(
        "assets/create/models/block/shared.json".into(),
        json!({
            "parent": "minecraft:block/block", "textures": {"all": "create:block/test", "particle": "#all"},
            "elements": [{"from": [0, 0, 0], "to": [16, 16, 16], "faces": {"north": {"texture": "#all"}}}]
        }),
    );
    json_file(
        "assets/create/lang/en_us.json".into(),
        json!({"block.create.shaft":"Fixture Shaft"}),
    );
    json_file(
        "assets/create/lang/ru_ru.json".into(),
        json!({"block.create.shaft":"Фикстура"}),
    );
    json_file(
        "assets/create/sounds.json".into(),
        json!({"fixture":{"sounds":["create:fixture"]}}),
    );
    files.insert(
        "assets/create/sounds/fixture.ogg".into(),
        b"OggSsynthetic fixture".to_vec(),
    );
    files.insert(
        "assets/create/textures/block/test.png".into(),
        b"\x89PNG\r\n\x1a\nsynthetic fixture".to_vec(),
    );
    files.insert(
        "assets/create/textures/block/test.png.mcmeta".into(),
        b"{\"animation\":{\"frametime\":2}}".to_vec(),
    );
    files.insert(
        "LICENSE.md_create-1.21.1".into(),
        b"Synthetic fixture license".to_vec(),
    );
    files.insert(
        "example/Untrusted.class".into(),
        b"never executed or extracted".to_vec(),
    );
    files.insert(
        "assets/create/textures/block/unused.png".into(),
        b"unused".to_vec(),
    );
    files.insert(
        "data/create/recipe/unsafe.json".into(),
        b"not imported".to_vec(),
    );
    change(&mut files);
    let mut zip = ZipWriter::new(Cursor::new(Vec::new()));
    for (name, bytes) in files {
        zip.start_file(name, SimpleFileOptions::default()).unwrap();
        zip.write_all(&bytes).unwrap();
    }
    zip.finish().unwrap().into_inner()
}

fn import_fixture(home: &Path, bytes: &[u8]) -> io::Result<Imported> {
    let jar = home.join("input.jar");
    fs::write(&jar, bytes)?;
    import(
        &jar,
        &home.join("cache"),
        (97, 1),
        &format!("{:x}", Sha256::digest(bytes)),
    )
}

#[test]
fn imports_every_locale_byte_for_byte() {
    let bytes = fixture(|files| {
        files.insert(
            "assets/create/lang/test_test.json".into(),
            b"{ \"block.create.shaft\" : \"Exact whitespace\" }\n".to_vec(),
        );
    });
    let files = collect(bytes.clone()).unwrap();
    let mut archive = ZipArchive::new(Cursor::new(bytes)).unwrap();
    for locale in ["en_us", "ru_ru", "test_test"] {
        let path = format!("assets/create/lang/{locale}.json");
        let mut expected = Vec::new();
        archive
            .by_name(&path)
            .unwrap()
            .read_to_end(&mut expected)
            .unwrap();
        assert_eq!(files[&path], expected);
    }
}

#[test]
fn processor_parts_and_ingredients_are_required_without_accepting_java_loaders() {
    let home = Stage::new(&std::env::temp_dir()).unwrap();
    let required = [
        "assets/create/blockstates/millstone.json",
        "assets/create/blockstates/encased_fan.json",
        "assets/create/models/block/millstone/inner.json",
        "assets/create/models/block/encased_fan/propeller.json",
        "assets/create/models/item/propeller.json",
        "assets/create/models/item/wheat_flour.json",
        "assets/create/models/item/dough.json",
        "assets/create/blockstates/mechanical_drill.json",
        "assets/create/models/block/mechanical_drill/head.json",
        "assets/create/models/item/mechanical_drill.json",
    ];
    for path in required {
        let missing = fixture(|files| {
            assert!(files.remove(path).is_some(), "catalog must require {path}");
        });
        assert!(import_fixture(&home.0, &missing).is_err(), "{path}");
        assert!(!home.0.join("cache").exists());
        assert_eq!(fs::read(home.0.join("input.jar")).unwrap(), missing);
    }
    for path in [
        "assets/create/models/block/millstone/inner.json",
        "assets/create/models/block/encased_fan/propeller.json",
        "assets/create/models/block/mechanical_drill/head.json",
    ] {
        let java = fixture(|files| {
            files.insert(path.into(), br#"{"loader":"create:java_loader"}"#.to_vec());
        });
        let error = import_fixture(&home.0, &java).unwrap_err();
        assert!(error.to_string().contains("Custom Java model loaders"));
        assert!(!home.0.join("cache").exists());
    }
}

#[test]
fn processor_catalog_uses_a_new_cache_schema() {
    let home = Stage::new(&std::env::temp_dir()).unwrap();
    let bytes = fixture(|_| {});
    let first = import_fixture(&home.0, &bytes).unwrap();
    let target = first.pack.parent().unwrap();
    let old = target.with_file_name(target.file_name().unwrap().to_str().unwrap().replacen(
        &format!("v{SCHEMA}-"),
        &format!("v{}-", SCHEMA - 1),
        1,
    ));
    let mut receipt: Value =
        serde_json::from_slice(&fs::read(target.join("import.json")).unwrap()).unwrap();
    receipt["schema"] = json!(SCHEMA - 1);
    fs::write(
        target.join("import.json"),
        serde_json::to_vec(&receipt).unwrap(),
    )
    .unwrap();
    fs::remove_file(
        first
            .pack
            .join("assets/create/models/block/mechanical_drill/head.json"),
    )
    .unwrap();
    fs::rename(target, &old).unwrap();
    let upgraded = import_fixture(&home.0, &bytes).unwrap();
    assert!(!upgraded.cached);
    assert!(
        upgraded
            .pack
            .join("assets/create/models/block/millstone/inner.json")
            .is_file()
    );
    assert!(
        upgraded
            .pack
            .join("assets/create/models/block/encased_fan/propeller.json")
            .is_file()
    );
    assert!(
        upgraded
            .pack
            .join("assets/create/models/block/mechanical_drill/head.json")
            .is_file()
    );
    assert!(old.is_dir());
    assert!(import_fixture(&home.0, &bytes).unwrap().cached);
}

#[test]
fn bundled_processors_have_source_states_collision_and_real_processing_recipes() {
    let bundle: BTreeMap<String, String> =
        serde_json::from_str(include_str!("../../data/content.json")).unwrap();
    let read = |path: &str| -> Value { serde_json::from_str(&bundle[path]).unwrap() };
    let millstone = read("data/create/horizon/block/millstone.json");
    assert!(millstone.get("state_properties").is_none());
    assert!(millstone.get("default_state").is_none());
    assert_eq!(
        millstone["collision_shapes"],
        json!({"default":[[0,0,0,1,0.375,1],[0.125,0.375,0.125,0.875,1,0.875]]})
    );
    assert_eq!(millstone["shapes"], millstone["collision_shapes"]);
    let fan = read("data/create/horizon/block/encased_fan.json");
    assert_eq!(fan["state_properties"].as_object().unwrap().len(), 1);
    assert_eq!(fan["default_state"], json!({"facing":"down"}));
    assert_eq!(
        fan["state_properties"]["facing"]["values"],
        json!(["down", "up", "north", "south", "west", "east"])
    );
    assert_eq!(fan["collision_shapes"], json!({"default":[[0,0,0,1,1,1]]}));
    let drill = read("data/create/horizon/block/mechanical_drill.json");
    assert_eq!(
        drill["default_state"],
        json!({"facing":"down","waterlogged":"false"})
    );
    assert_eq!(drill["shapes"], drill["collision_shapes"]);
    assert_eq!(drill["collision_shapes"]["property"], "facing");
    assert_eq!(
        drill["collision_shapes"]["values"]["up"],
        json!([[0, 0, 0, 1, 0.75, 1]])
    );
    assert_eq!(
        drill["collision_shapes"]["values"]["east"],
        json!([[0, 0, 0, 0.75, 1, 1]])
    );
    assert_eq!(
        read("assets/create/blockstates/mechanical_drill.json")["variants"]
            .as_object()
            .unwrap()
            .len(),
        12
    );
    assert_eq!(
        read("data/create/recipe/crafting/kinetics/mechanical_drill.json")["result"]["id"],
        "create:mechanical_drill"
    );
    for id in ["millstone", "encased_fan", "mechanical_drill"] {
        let loot = read(&format!("data/create/loot_table/blocks/{id}.json"));
        assert_eq!(
            loot["pools"][0]["entries"][0]["name"],
            format!("create:{id}")
        );
    }
    let wheat = read("data/create/recipe/milling/wheat.json");
    assert_eq!(wheat["type"], "create:milling");
    assert_eq!(wheat["processing_time"], 150);
    assert_eq!(
        wheat["results"],
        json!([{"id":"create:wheat_flour"},{"chance":0.25,"count":2,"id":"create:wheat_flour"},{"chance":0.25,"id":"minecraft:wheat_seeds"}])
    );
    let dough = read("data/create/recipe/splashing/wheat_flour.json");
    assert_eq!(dough["ingredients"], json!(["#c:flours/wheat"]));
    assert_eq!(dough["results"], json!([{"id":"create:dough"}]));
    for kind in ["smelting", "smoking"] {
        let bread = read(&format!("data/create/recipe/{kind}/bread.json"));
        assert_eq!(bread["type"], format!("minecraft:{kind}"));
        assert_eq!(bread["ingredient"], "create:dough");
        assert_eq!(bread["result"]["id"], "minecraft:bread");
    }
    assert_eq!(
        read("data/create/recipe/haunting/soul_sand.json")["ingredients"],
        json!(["#minecraft:sand"])
    );
    assert_eq!(
        read("data/create/recipe/milling/wool.json")["results"][0]["id"],
        "minecraft:string"
    );
    assert_eq!(
        read("data/create/recipe/milling/bone.json")["results"]
            .as_array()
            .unwrap()
            .len(),
        3
    );
    for (path, values) in [
        (
            "c:glass_blocks",
            vec!["minecraft:glass", "minecraft:red_stained_glass"],
        ),
        (
            "c:glass_panes",
            vec!["minecraft:glass_pane", "minecraft:red_stained_glass_pane"],
        ),
        ("c:stones", vec!["minecraft:stone", "minecraft:andesite"]),
        ("c:flours/wheat", vec!["create:wheat_flour"]),
    ] {
        let (namespace, path) = path.split_once(':').unwrap();
        let tag = read(&format!("data/{namespace}/tags/item/{path}.json"));
        for value in values {
            assert!(
                tag["values"].as_array().unwrap().contains(&json!(value)),
                "{path}: {value}"
            );
        }
    }
    let transparent = read("data/create/tags/block/fan_transparent.json");
    assert!(
        transparent["values"]
            .as_array()
            .unwrap()
            .contains(&json!("minecraft:iron_bars"))
    );
    assert!(
        transparent["values"]
            .as_array()
            .unwrap()
            .contains(&json!("#minecraft:fences"))
    );
    assert!(
        !transparent["values"]
            .as_array()
            .unwrap()
            .iter()
            .any(|v| v.as_str().unwrap().starts_with("create:"))
    );
    assert_eq!(
        read("data/create/tags/block/fan_processing_catalysts/blasting.json")["values"],
        json!([])
    );
    assert_eq!(
        read("data/create/tags/block/fan_processing_catalysts/haunting.json")["values"],
        json!(["minecraft:soul_fire", "minecraft:soul_campfire"])
    );
    assert_eq!(
        read("data/create/tags/block/fan_processing_catalysts/smoking.json")["values"],
        json!(["minecraft:fire", "minecraft:campfire"])
    );
    assert_eq!(
        read("data/create/tags/fluid/fan_processing_catalysts/splashing.json")["values"],
        json!(["minecraft:water", "minecraft:flowing_water"])
    );
    assert_eq!(
        read("data/create/tags/fluid/fan_processing_catalysts/blasting.json")["values"],
        json!(["minecraft:lava", "minecraft:flowing_lava"])
    );
}

#[test]
fn bundled_content_repairs_itself_without_changing_other_mods_or_settings() {
    let home = std::env::temp_dir().join(format!("create-content-{}", std::process::id()));
    let _ = fs::remove_dir_all(&home);
    fs::create_dir_all(home.join("horizon-mods")).unwrap();
    fs::write(
        home.join("horizon-mods.txt"),
        "mod.create:true\nmod.other:false\n",
    )
    .unwrap();
    fs::write(home.join("horizon-mods/other.hmod"), b"another mod").unwrap();
    let bytes = serde_json::to_vec(&json!({
        "horizon-mod.toml": "id = \"create\"\nversion = \"0.1.0\"\n",
        "assets/create/items/shaft.json": "{\"model\": {}}"
    }))
    .unwrap();
    let path = bundled_content(&home, &bytes).unwrap();
    fs::write(path.join("assets/create/items/shaft.json"), "damaged").unwrap();
    fs::write(path.join("stale-file"), "old release").unwrap();
    assert_eq!(bundled_content(&home, &bytes).unwrap(), path);
    assert_eq!(
        fs::read_to_string(path.join("assets/create/items/shaft.json")).unwrap(),
        "{\"model\": {}}"
    );
    assert!(!path.join("stale-file").exists());
    let modified = fs::metadata(path.join("horizon-mod.toml"))
        .unwrap()
        .modified()
        .unwrap();
    bundled_content(&home, &bytes).unwrap();
    assert_eq!(
        fs::metadata(path.join("horizon-mod.toml"))
            .unwrap()
            .modified()
            .unwrap(),
        modified
    );
    assert_eq!(
        fs::read(home.join("horizon-mods/other.hmod")).unwrap(),
        b"another mod"
    );
    assert_eq!(
        fs::read_to_string(home.join("horizon-mods.txt")).unwrap(),
        "mod.create:true\nmod.other:false\n"
    );
    fs::remove_dir_all(home).unwrap();
}

#[test]
fn bundled_content_rejects_paths_outside_its_namespace_before_writing() {
    let home = std::env::temp_dir().join(format!("create-content-invalid-{}", std::process::id()));
    for name in [
        "../escape",
        "assets/minecraft/override.json",
        "data/create/../../escape",
        "/absolute",
    ] {
        let bytes = serde_json::to_vec(&BTreeMap::from([
            ("horizon-mod.toml", "id = \"create\""),
            (name, "invalid"),
        ]))
        .unwrap();
        assert!(bundled_content(&home, &bytes).is_err());
    }
    assert!(!home.exists());
}

#[test]
fn imports_dependencies_metadata_and_notice_without_code_or_unused_assets() {
    let home = Stage::new(&std::env::temp_dir()).unwrap();
    let bytes = fixture(|_| {});
    let imported = import_fixture(&home.0, &bytes).unwrap();
    assert!(!imported.cached);
    assert_eq!(fs::read(home.0.join("input.jar")).unwrap(), bytes);
    for file in [
        "assets/create/models/block/shared.json",
        "assets/create/textures/block/test.png",
        "assets/create/textures/block/test.png.mcmeta",
        "assets/create/sounds.json",
        "assets/create/sounds/fixture.ogg",
        "CREATE-LICENSE.md",
        "pack.mcmeta",
    ] {
        assert!(imported.pack.join(file).is_file(), "{file}");
    }
    for file in [
        "example/Untrusted.class",
        "assets/create/textures/block/unused.png",
        "data/create/recipe/unsafe.json",
    ] {
        assert!(!imported.pack.join(file).exists(), "{file}");
    }
    let metadata: Value =
        serde_json::from_slice(&fs::read(imported.pack.join("pack.mcmeta")).unwrap()).unwrap();
    assert_eq!(metadata["pack"]["min_format"], json!([97, 1]));
}

#[test]
fn cache_is_reused_and_missing_or_changed_resources_are_repaired() {
    let home = Stage::new(&std::env::temp_dir()).unwrap();
    let bytes = fixture(|_| {});
    let first = import_fixture(&home.0, &bytes).unwrap();
    assert!(import_fixture(&home.0, &bytes).unwrap().cached);
    let texture = first.pack.join("assets/create/textures/block/test.png");
    let expected = fs::read(&texture).unwrap();
    fs::write(&texture, b"corrupt").unwrap();
    assert!(!import_fixture(&home.0, &bytes).unwrap().cached);
    assert_eq!(fs::read(&texture).unwrap(), expected);
    fs::remove_file(&texture).unwrap();
    assert!(!import_fixture(&home.0, &bytes).unwrap().cached);
    assert!(texture.is_file());
    fs::write(first.pack.join("pack.mcmeta"), b"corrupt").unwrap();
    assert!(!import_fixture(&home.0, &bytes).unwrap().cached);
    assert!(
        serde_json::from_slice::<Value>(&fs::read(first.pack.join("pack.mcmeta")).unwrap()).is_ok()
    );
}

#[test]
fn missing_jar_creates_instructions_and_ignores_old_cache() {
    let home = Stage::new(&std::env::temp_dir()).unwrap();
    import_fixture(&home.0, &fixture(|_| {})).unwrap();
    assert!(prepare(&home.0, (97, 1)).unwrap().is_none());
    assert!(home.0.join(INPUT_DIR).join("README.txt").is_file());
}

#[test]
fn ambiguous_jars_and_wrong_hash_are_rejected_without_altering_input() {
    let home = Stage::new(&std::env::temp_dir()).unwrap();
    let input = home.0.join(INPUT_DIR);
    fs::create_dir_all(&input).unwrap();
    let bytes = fixture(|_| {});
    fs::write(input.join("a.jar"), &bytes).unwrap();
    fs::write(input.join("b.JAR"), &bytes).unwrap();
    assert!(
        prepare(&home.0, (97, 1))
            .unwrap_err()
            .to_string()
            .contains("exactly one")
    );
    fs::remove_file(input.join("b.JAR")).unwrap();
    assert!(
        prepare(&home.0, (97, 1))
            .unwrap_err()
            .to_string()
            .contains("Unsupported Create")
    );
    assert_eq!(fs::read(input.join("a.jar")).unwrap(), bytes);
    assert!(!home.0.join(".horizon-cache/create-assets/imports").exists());
}

#[test]
fn traversal_missing_dependencies_parent_cycles_and_java_loaders_fail_before_publish() {
    let home = Stage::new(&std::env::temp_dir()).unwrap();
    let variants = [
        fixture(|files| {
            files.insert("../escape".into(), b"bad".to_vec());
        }),
        fixture(|files| {
            files.remove("assets/create/textures/block/test.png");
        }),
        fixture(|files| {
            files.insert(
                "assets/create/models/block/shared.json".into(),
                b"{\"parent\":\"create:block/shared\"}".to_vec(),
            );
        }),
        fixture(|files| {
            files.insert(
                "assets/create/models/block/shared.json".into(),
                b"{\"loader\":\"create:java_loader\"}".to_vec(),
            );
        }),
        fixture(|files| {
            files.insert(
                "assets/create/models/block/shared.json".into(),
                b"{\"parent\":\"create:../../escape\"}".to_vec(),
            );
        }),
        fixture(|files| {
            files.insert(
                "assets/create/models/block/shared.json".into(),
                b"invalid json".to_vec(),
            );
        }),
    ];
    for bytes in variants {
        assert!(import_fixture(&home.0, &bytes).is_err());
        assert!(!home.0.join("cache").exists());
        assert!(!home.0.parent().unwrap().join("escape").exists());
    }
}

#[test]
fn oversized_resources_are_refused() {
    let home = Stage::new(&std::env::temp_dir()).unwrap();
    let bytes = fixture(|files| {
        files.insert(
            "assets/create/textures/block/test.png".into(),
            vec![0; MAX_FILE as usize + 1],
        );
    });
    assert!(
        import_fixture(&home.0, &bytes)
            .unwrap_err()
            .to_string()
            .contains("oversized")
    );
    assert!(!home.0.join("cache").exists());
}

#[cfg(unix)]
#[test]
fn symlinked_input_is_not_followed() {
    let home = Stage::new(&std::env::temp_dir()).unwrap();
    fs::write(home.0.join("elsewhere.jar"), fixture(|_| {})).unwrap();
    fs::create_dir_all(home.0.join(INPUT_DIR)).unwrap();
    std::os::unix::fs::symlink(
        home.0.join("elsewhere.jar"),
        home.0.join(INPUT_DIR).join("input.jar"),
    )
    .unwrap();
    assert!(
        prepare(&home.0, (97, 1))
            .unwrap_err()
            .to_string()
            .contains("regular file")
    );
}

#[test]
fn sound_bytes_are_preserved_and_invalid_ogg_is_rejected() {
    let home = Stage::new(&std::env::temp_dir()).unwrap();
    let imported = import_fixture(&home.0, &fixture(|_| {})).unwrap();
    assert_eq!(
        fs::read(imported.pack.join("assets/create/sounds/fixture.ogg")).unwrap(),
        b"OggSsynthetic fixture"
    );
    let invalid = fixture(|files| {
        files.insert(
            "assets/create/sounds/fixture.ogg".into(),
            b"invalid audio".to_vec(),
        );
    });
    assert!(import_fixture(&home.0, &invalid).is_err());
}

#[test]
fn obj_models_retain_geometry_and_safe_materials_without_java() {
    let home = Stage::new(&std::env::temp_dir()).unwrap();
    let bytes = fixture(|files| {
        files.insert("assets/create/models/block/shared.json".into(), br##"{"loader":"neoforge:obj","model":"create:models/block/shape.obj","textures":{"all":"create:block/test"}}"##.to_vec());
        files.insert(
            "assets/create/models/block/shape.obj".into(),
            b"mtllib shape.mtl\nv 0 0 0\nv 0 1 0\nv 1 1 0\nv 1 0 0\nf 1 2 3 4\n".to_vec(),
        );
        files.insert(
            "assets/create/models/block/shape.mtl".into(),
            b"newmtl shape\nmap_Kd #all\n".to_vec(),
        );
    });
    let imported = import_fixture(&home.0, &bytes).unwrap();
    assert!(
        fs::read_to_string(imported.pack.join("assets/create/models/block/shape.obj"))
            .unwrap()
            .contains("f 1 2 3 4")
    );
    let hostile = fixture(|files| {
        files.insert(
            "assets/create/models/block/shared.json".into(),
            br##"{"loader":"neoforge:obj","model":"create:models/block/../../shape.obj"}"##
                .to_vec(),
        );
    });
    assert!(import_fixture(&home.0, &hostile).is_err());
}
