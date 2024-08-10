use anyhow::Result;
use assert_fs::prelude::*;
#[cfg(test)]
use pretty_assertions::{assert_eq, assert_ne};

use tsp::{cli::BuildCommand, config};

#[test]
fn current_from_generated_default() -> Result<()> {
    let temp = assert_fs::TempDir::new()?;
    let generated = temp.child("generated.toml");
    let def = BuildCommand::default();
    generated.write_str(&toml::to_string(&def)?)?;
    assert_eq!(def, config::current(&generated, None).unwrap());
    Ok(())
}

#[test]
fn current_from_empty() -> Result<()> {
    let temp = assert_fs::TempDir::new()?;
    let generated = temp.child("generated.toml");
    let def = BuildCommand::default();
    generated.touch()?;
    assert_eq!(def, config::current(&generated, None).unwrap());
    Ok(())
}

#[test]
fn current_preserve_languages() -> Result<()> {
    let temp = assert_fs::TempDir::new()?;
    let generated = temp.child("generated.toml");
    let mut def = BuildCommand::default();
    generated.touch()?;
    def.languages = None;
    assert_eq!(def, config::current(&generated, Some(&def)).unwrap());
    def.languages = Some(vec![]);
    assert_eq!(def, config::current(&generated, Some(&def)).unwrap());
    def.languages = Some(vec!["rust".to_string()]);
    assert_eq!(def, config::current(&generated, Some(&def)).unwrap());
    def.languages = Some(vec!["rust".to_string(), "ruby".to_string()]);
    assert_eq!(def, config::current(&generated, Some(&def)).unwrap());
    Ok(())
}

#[test]
fn current_default_is_default() -> Result<()> {
    let config = r#"
    build-dir = "tmp"
    static = false
    fresh = false
    out = "parsers"
    show-config = false

    [tree-sitter]
    version = "0.22.6"
    repo = "https://github.com/tree-sitter/tree-sitter"
    platform = "macos-arm64"
  "#;
    let temp = assert_fs::TempDir::new()?;
    let generated = temp.child("generated.toml");
    let def = BuildCommand::default();
    generated.write_str(config)?;
    assert_eq!(def, config::current(&generated, None).unwrap());
    assert_eq!(def, config::current(&generated, Some(&def)).unwrap());
    Ok(())
}

#[test]
fn current_overrides_default() -> Result<()> {
    let config = r#"
      build-dir = "/root"
      static = true
      fresh = true
      out = "tree-sitter-parsers"
      show-config = true

      [tree-sitter]
      version = "1.0.0"
      repo = "https://gitlab.com/tree-sitter/tree-sitter"
      platform = "linux-arm64"
    "#;
    let temp = assert_fs::TempDir::new()?;
    let generated = temp.child("generated.toml");
    let def = BuildCommand::default();
    generated.write_str(config)?;
    generated.assert(config);
    assert_ne!(def, config::current(&generated, None).unwrap());
    assert_ne!(def, config::current(&generated, Some(&def)).unwrap());
    Ok(())
}
