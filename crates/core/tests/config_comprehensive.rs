use novalyn_core::config::{LoadOptions, SemverImpact, default_types, load_config};
use std::fs;
use tempfile::TempDir;

#[test]
fn test_load_config_with_novalyn_toml() {
    let dir = TempDir::new().unwrap();
    let config_path = dir.path().join("novalyn.toml");

    fs::write(
        &config_path,
        r#"
[types.feat]
title = "New Features"
emoji = "✨"
semver = "minor"

[types.fix]
title = "Bug Fixes"
emoji = "🐛"
semver = "patch"
"#,
    )
    .unwrap();

    let cfg = load_config(LoadOptions {
        cwd: dir.path(),
        cli_overrides: None,
    })
    .unwrap();

    assert!(cfg.source_file.is_some());
    let feat_type = cfg.types.iter().find(|t| t.key == "feat").unwrap();
    assert_eq!(feat_type.title, "New Features");
    assert_eq!(feat_type.emoji, "✨");
    assert_eq!(feat_type.semver, SemverImpact::Minor);
}

#[test]
fn test_load_config_with_cargo_metadata() {
    let dir = TempDir::new().unwrap();
    let cargo_path = dir.path().join("Cargo.toml");

    fs::write(
        &cargo_path,
        r#"
[package]
name = "test"
version = "0.1.0"

[package.metadata.novalyn]
new_version = "1.0.0"

[package.metadata.novalyn.types.feat]
emoji = "🎉"
"#,
    )
    .unwrap();

    let cfg = load_config(LoadOptions {
        cwd: dir.path(),
        cli_overrides: None,
    })
    .unwrap();

    assert!(cfg.new_version.is_some());
    assert_eq!(cfg.new_version.unwrap().to_string(), "1.0.0");
}

#[test]
fn test_load_config_disable_type() {
    let dir = TempDir::new().unwrap();
    let config_path = dir.path().join("novalyn.toml");

    fs::write(
        &config_path,
        r#"
[types]
chore = false
"#,
    )
    .unwrap();

    let cfg = load_config(LoadOptions {
        cwd: dir.path(),
        cli_overrides: None,
    })
    .unwrap();

    let chore_type = cfg.types.iter().find(|t| t.key == "chore");
    assert!(chore_type.is_some());
    assert!(!chore_type.unwrap().enabled);
}

#[test]
fn test_load_config_scope_map() {
    let dir = TempDir::new().unwrap();
    let config_path = dir.path().join("novalyn.toml");

    fs::write(
        &config_path,
        r#"
[scope_map]
api = "API"
ui = "UI"
old_scope = ""
"#,
    )
    .unwrap();

    let cfg = load_config(LoadOptions {
        cwd: dir.path(),
        cli_overrides: None,
    })
    .unwrap();

    assert_eq!(cfg.scope_map.get("api").map(|s| s.as_str()), Some("API"));
    assert_eq!(cfg.scope_map.get("ui").map(|s| s.as_str()), Some("UI"));
    assert_eq!(cfg.scope_map.get("old_scope").map(|s| s.as_str()), Some(""));
}

#[test]
fn test_default_types_complete() {
    let types = default_types();

    // Check that all standard types are present
    let type_keys: Vec<_> = types.iter().map(|t| t.key.as_str()).collect();
    assert!(type_keys.contains(&"feat"));
    assert!(type_keys.contains(&"fix"));
    assert!(type_keys.contains(&"docs"));
    assert!(type_keys.contains(&"style"));
    assert!(type_keys.contains(&"refactor"));
    assert!(type_keys.contains(&"perf"));
    assert!(type_keys.contains(&"test"));
    assert!(type_keys.contains(&"build"));
    assert!(type_keys.contains(&"ci"));
    assert!(type_keys.contains(&"chore"));

    // Check semver impacts
    let feat = types.iter().find(|t| t.key == "feat").unwrap();
    assert_eq!(feat.semver, SemverImpact::Minor);

    let fix = types.iter().find(|t| t.key == "fix").unwrap();
    assert_eq!(fix.semver, SemverImpact::Patch);

    let docs = types.iter().find(|t| t.key == "docs").unwrap();
    assert_eq!(docs.semver, SemverImpact::None);
}

#[test]
fn test_load_config_warnings() {
    let dir = TempDir::new().unwrap();
    let config_path = dir.path().join("novalyn.toml");

    // Invalid new_version
    fs::write(
        &config_path,
        r#"
new_version = "invalid-version"
unknown_key = "value"
"#,
    )
    .unwrap();

    let cfg = load_config(LoadOptions {
        cwd: dir.path(),
        cli_overrides: None,
    })
    .unwrap();

    assert!(!cfg.warnings.is_empty());
    let warnings_str = cfg.warnings.join(", ");
    assert!(warnings_str.contains("Invalid new_version"));
    assert!(warnings_str.contains("unknown_key"));
}

#[test]
fn test_config_precedence() {
    let dir = TempDir::new().unwrap();
    let config_path = dir.path().join("novalyn.toml");

    fs::write(
        &config_path,
        r#"
[types.feat]
title = "File Config"
"#,
    )
    .unwrap();

    let cli_override = novalyn_core::config::RawConfig {
        types_override: Some(
            [(
                "feat".into(),
                novalyn_core::config::TypeToggleOrConfig::Config(
                    novalyn_core::config::TypeConfigPartial {
                        title: Some("CLI Override".into()),
                        emoji: None,
                        semver: None,
                    },
                ),
            )]
            .into_iter()
            .collect(),
        ),
        ..Default::default()
    };

    let cfg = load_config(LoadOptions {
        cwd: dir.path(),
        cli_overrides: Some(cli_override),
    })
    .unwrap();

    let feat_type = cfg.types.iter().find(|t| t.key == "feat").unwrap();
    // CLI override should win
    assert_eq!(feat_type.title, "CLI Override");
}
