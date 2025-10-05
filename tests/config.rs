use std::fs;

use novalyn::config::{self, LoadOptions, RawConfig};
use insta::assert_yaml_snapshot;

fn temp_dir() -> tempfile::TempDir {
    tempfile::tempdir().unwrap()
}

#[test]
fn default_config_snapshot() {
    let dir = temp_dir();
    let cfg = config::load_config(LoadOptions {
        cwd: dir.path(),
        cli_overrides: None,
    })
    .unwrap();
    assert_yaml_snapshot!(
        "default_config",
        cfg.types
            .iter()
            .map(|t| (
                &t.key,
                &t.title,
                &t.emoji,
                format!("{:?}", t.semver),
                t.enabled
            ))
            .collect::<Vec<_>>()
    );
}

#[test]
fn precedence_cli_over_file() {
    let dir = temp_dir();
    // write novalyn.toml overriding feat title
    fs::write(
        dir.path().join("novalyn.toml"),
        "[types.feat]\ntitle='Features A'\n",
    )
    .unwrap();
    let cli = RawConfig {
        types_override: Some({
            let mut m = std::collections::BTreeMap::new();
            m.insert(
                "feat".into(),
                config::TypeToggleOrConfig::Config(config::TypeConfigPartial {
                    title: Some("Features B".into()),
                    emoji: None,
                    semver: None,
                }),
            );
            m
        }),
        ..Default::default()
    };
    let cfg = config::load_config(LoadOptions {
        cwd: dir.path(),
        cli_overrides: Some(cli),
    })
    .unwrap();
    let feat = cfg.types.iter().find(|t| t.key == "feat").unwrap();
    assert_eq!(feat.title, "Features B");
}

#[test]
fn disabling_type_boolean_false() {
    let dir = temp_dir();
    fs::write(dir.path().join("novalyn.toml"), "[types]\nfeat=false\n").unwrap();
    let cfg = config::load_config(LoadOptions {
        cwd: dir.path(),
        cli_overrides: None,
    })
    .unwrap();
    let feat = cfg.types.iter().find(|t| t.key == "feat").unwrap();
    assert!(!feat.enabled);
}

#[test]
fn env_token_precedence() {
    let dir = temp_dir();
    // set lower precedence tokens first
    unsafe {
        std::env::remove_var("NOVALYN_TOKENS_GITHUB");
        std::env::set_var("GH_TOKEN", "gh_low");
        std::env::set_var("GITHUB_TOKEN", "gh_mid");
    }
    let cfg_mid = config::load_config(LoadOptions {
        cwd: dir.path(),
        cli_overrides: None,
    })
    .unwrap();
    assert_eq!(cfg_mid.github_token.as_deref(), Some("gh_mid"));
    unsafe {
        std::env::set_var("NOVALYN_TOKENS_GITHUB", "gh_high");
    }
    let cfg_high = config::load_config(LoadOptions {
        cwd: dir.path(),
        cli_overrides: None,
    })
    .unwrap();
    assert_eq!(cfg_high.github_token.as_deref(), Some("gh_high"));
}
