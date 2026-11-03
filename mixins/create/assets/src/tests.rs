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
    for item in BLOCKS.into_iter().chain(SHEETS) {
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
