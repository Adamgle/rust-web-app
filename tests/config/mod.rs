use rust_web_app::config::Env;

#[test]
fn test_enum_file_equality() -> anyhow::Result<()> {
    let file_envs = Env::get_file_envs()?;
    let enum_envs = Env::get_enum_envs()?;

    assert_eq!(file_envs, enum_envs);

    Ok(())
}
