use mc_resources::{
    asset::AssetIndex,
    manifest::{VersionManifest, VersionType},
    version::VersionInfo,
};

const SHA1: &str = "a9993e364706816aba3e25717850c26c9cd0d89d";

fn version_manifest_json() -> String {
    format!(
        r#"{{"latest":{{"release":"1.0","snapshot":"snapshot-1"}},"versions":[
          {{"id":"1.0","type":"release","url":"https://example.invalid/1.json","time":"2020-01-01T00:00:00+00:00","releaseTime":"2020-01-01T00:00:00+00:00","sha1":"{SHA1}","complianceLevel":1}},
          {{"id":"snapshot-1","type":"snapshot","url":"https://example.invalid/snapshot.json","time":"2020-01-02T00:00:00+00:00","releaseTime":"2020-01-02T00:00:00+00:00","sha1":"{SHA1}","complianceLevel":1}}
        ]}}"#
    )
}

fn version_info_json(arguments_field: &str) -> String {
    format!(
        r#"{{
          "{arguments_field}":"--username ${{auth_player_name}} --version ${{version_name}}",
          "assetIndex":{{"id":"1","sha1":"{SHA1}","size":10,"totalSize":20,"url":"https://example.invalid/assets.json"}},
          "assets":"1","downloads":{{"client":{{"sha1":"{SHA1}","size":3,"url":"https://example.invalid/client.jar"}}}},
          "id":"1.0","libraries":[],"mainClass":"example.Main","minimumLauncherVersion":21,
          "releaseTime":"2020-01-01T00:00:00+00:00","time":"2020-01-01T00:00:00+00:00","type":"release"
        }}"#
    )
}

#[test]
fn version_manifest_supports_latest_lookup_and_find() {
    let manifest: VersionManifest = serde_json::from_str(&version_manifest_json()).unwrap();
    assert_eq!(
        manifest.latest_release().unwrap().version_type,
        VersionType::Release
    );
    assert_eq!(manifest.latest_snapshot().unwrap().id, "snapshot-1");
    assert_eq!(manifest.find_version("1.0").unwrap().id, "1.0");
    assert!(manifest.find_version("missing").is_none());
    assert_eq!(manifest.versions().len(), 2);
}

#[test]
fn asset_index_parses_and_looks_up_named_objects() {
    let index: AssetIndex = serde_json::from_str(&format!(
        r#"{{"objects":{{"minecraft/sounds/test.ogg":{{"hash":"{SHA1}","size":3}}}}}}"#
    ))
    .unwrap();
    let object = index.get_object("minecraft/sounds/test.ogg").unwrap();
    assert_eq!(object.hash.to_string(), SHA1);
    assert_eq!(object.size, 3);
    assert_eq!(index.objects().count(), 1);
    assert_eq!(
        index.assets().next().unwrap().0,
        "minecraft/sounds/test.ogg"
    );
    assert!(index.get_object("missing").is_none());
}

#[test]
fn version_info_accepts_legacy_alias_and_structured_arguments() {
    let legacy: VersionInfo =
        serde_json::from_str(&version_info_json("minecraftArguments")).unwrap();
    assert!(matches!(
        legacy.arguments,
        Some(mc_resources::version::args::CompatibleArguments::Text(_))
    ));

    let structured_json = version_info_json("arguments").replace(
        "\"--username ${auth_player_name} --version ${version_name}\"",
        r#"{"game":["--username"],"jvm":[]}"#,
    );
    let structured: VersionInfo = serde_json::from_str(&structured_json).unwrap();
    assert!(matches!(
        structured.arguments,
        Some(mc_resources::version::args::CompatibleArguments::Structured(_))
    ));
    assert_eq!(structured.downloads.client.size, 3);
    assert!(structured.java_version.is_none());
}

#[test]
fn optional_manifest_fields_may_be_absent() {
    let info: VersionInfo = serde_json::from_str(&version_info_json("minecraftArguments")).unwrap();
    assert!(info.compliance_level.is_none());
    assert!(info.java_version.is_none());
    assert!(info.logging.is_none());
    assert!(info.downloads.server.is_none());
}

#[test]
fn malformed_models_fail_deserialization() {
    assert!(serde_json::from_str::<VersionManifest>(r#"{"latest":{},"versions":[]}"#).is_err());
    assert!(
        serde_json::from_str::<AssetIndex>(r#"{"objects":{"x":{"hash":"bad","size":1}}}"#).is_err()
    );
    assert!(
        serde_json::from_str::<AssetIndex>(
            r#"{"objects":{"x":{"hash":"a9993e364706816aba3e25717850c26c9cd0d89d","size":"big"}}}"#
        )
        .is_err()
    );
    assert!(
        serde_json::from_str::<VersionInfo>(
            &version_info_json("minecraftArguments")
                .replace("https://example.invalid/client.jar", "not a URL",)
        )
        .is_err()
    );
    assert!(
        serde_json::from_str::<VersionManifest>(
            &version_manifest_json().replace("\"type\":\"release\"", "\"type\":\"future_type\"",)
        )
        .is_err()
    );
}

#[test]
fn unknown_fields_follow_debug_feature_policy() {
    let with_unknown =
        version_manifest_json().replace("\"latest\":", "\"newField\":true,\"latest\":");
    let result = serde_json::from_str::<VersionManifest>(&with_unknown);
    if cfg!(feature = "debug") {
        assert!(result.is_err());
    } else {
        assert!(result.is_ok());
    }
}
