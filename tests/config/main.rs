// Integration tests for the config/mod.rs
// TODO: We need to figure out a better way for structuring the integration tests
// If I put the files in separate directories the analyzer does not link them.

use rust_web_app::config::Env;

#[test]
fn test_enum_file_equality() -> anyhow::Result<()> {
    let file_envs = Env::get_file_envs()?;
    let enum_envs = Env::get_enum_envs()?;

    assert_eq!(file_envs, enum_envs);

    Ok(())
}

#[test]
fn whatever() {
    assert_eq!(2 + 2, 4);
}
