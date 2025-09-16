mod error;
pub use self::error::{EnvError, Error};

use std::{collections::HashSet, fmt::Debug, hash::Hash, str::FromStr};
use strum::IntoEnumIterator;

pub(in crate::config) type Result<T> = std::result::Result<T, self::Error>;

pub struct Config;

/// ## This code should not happen. Written to practice unit testing.
///
/// Defines all the environment variables used in the application.
/// Throw runtime errors if there is any mismatch between the envs defined in the `.env` file
///
/// Denies non_camel_case_types to enforce the convention of enum variants.
///
/// Although that is strictly not necessary, since the serialize_all attribute serializes the variants to SCREAMING_SNAKE_CASE,
/// but that could produce unsound code if the variants are not following the convention.
///
/// Envs are validated to be in SCREAMING_SNAKE_CASE when loading from the `.env` file.
/// They are getting checked for duplicates after the translation to SCREAMING_SNAKE_CASE from serialize_all,
/// checked for declared, but non-existent, checked for missing envs define in the `.env` file.
#[derive(
    Clone,
    Debug,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    strum_macros::EnumIter,
    strum_macros::AsRefStr,
    strum_macros::Display,
    strum_macros::EnumString,
)]
#[strum(serialize_all = "SCREAMING_SNAKE_CASE")]
#[deny(non_camel_case_types)]
pub enum Env {
    ServerUrl,
    ServerPort,
    ClientUrl,
    ClientPort,
    DbAdminPostgresPassword,
    DbPostgresAdambPassword,
    DatabaseUrl,
}

impl Env {
    // TODO: Isolate to the config file, or ideally remove that shenanigans as backend should
    // not care about the frontend envs and they should be separate.
    pub const ENV_PATH: &str = ".env";

    /// Loads the envs from the `.env` file and checks for 1-1 mapping between the envs defined in the .env file and the enum variants.
    /// Tries to be detailed about the error messages converting the mismatches between the two.
    fn load_envs() -> self::Result<()> {
        // Converting to string allows the serialize_all attribute to kick in.
        // This way we can be sure that the envs are in SCREAMING_SNAKE_CASE and can be valid even when
        // not following the convention in the enum declaration.
        // FromStr does not take the serialize_all attribute into account, since the conversion to strings.

        Self::compare_envs(Self::get_file_envs()?)?;

        Ok(())
    }

    // The isolation between two function compare_envs and check_mapping is to make the
    // unit testing easier. Normally I would just call the check_mapping directly
    fn compare_envs(other: HashSet<String>) -> self::Result<()> {
        Self::check_mapping(Self::get_enum_envs()?, other)
    }

    /// Checks if there is 1-1 mapping between the envs declared in the .env file and the envs declared in the Env enum.
    ///
    /// Both are the same.
    fn check_mapping(enum_envs: HashSet<String>, file_envs: HashSet<String>) -> self::Result<()> {
        if file_envs != enum_envs {
            let missing_from_file = enum_envs
                .difference(&file_envs)
                .cloned()
                .collect::<HashSet<String>>();

            // The case of the missing from enum is not possible as the error would occur when converting with from_str
            if !missing_from_file.is_empty() {
                return Err(EnvError::MissingEnvFromFile(missing_from_file).into());
            }
        }

        Ok(())
    }

    /// Retrieves all the envs defined in the .env file and checks for duplicates and format.
    ///
    /// Make it visible for the integration tests.
    pub fn get_file_envs() -> self::Result<HashSet<String>> {
        let cwd = std::env::current_dir().map_err(EnvError::from)?;
        let env_path = cwd.join(Self::ENV_PATH);

        // NOTE: Maybe that approach would be better, read from some repo on github.
        // This returns an error if the `.env` file doesn't exist, but that's not what we want
        // since we're not going to use a `.env` file if we deploy this application
        dotenvy::from_path(env_path).map_err(EnvError::from)?;

        let file_envs = dotenvy::dotenv_iter().map_err(EnvError::from)?;
        let mut seen = HashSet::new();

        for env in file_envs {
            let (key, ..) = env.map_err(EnvError::from)?;

            // Since the key is in wrong format, surely there is not variant for it in the enum.
            // and we want to inform about that before we inform about the missing variant in the enum.
            if !key.chars().all(|r| r.is_uppercase() || r == '_') {
                return Err(EnvError::WrongFormat(key).into());
            }

            // If that conversion fails, that means that there is no variant for that env in the enum.
            let key = Env::from_str(&key)
                .map(|key: Env| key.to_string())
                .map_err(|_| EnvError::MissingEnvFromEnum(key))?;

            if !seen.insert(key.clone()) {
                return Err(EnvError::DuplicatedEnvInFile(key).into());
            }
        }

        Ok(seen)
    }

    /// Retrieves all the envs defined in the Env enum.
    ///
    /// Checks for duplicates after the translation to SCREAMING_SNAKE_CASE from serialize_all.
    pub fn get_enum_envs() -> self::Result<HashSet<String>> {
        Self::iter()
            .try_fold(HashSet::new(), |mut acc, env| {
                if !acc.insert(env.to_string()) {
                    Err(EnvError::DuplicatedEnvInEnum {
                        translation: env.to_string(),
                        variant: env,
                    })
                } else {
                    Ok(acc)
                }
            })
            .map_err(Error::from)
    }
}

impl Config {
    // NOTE: Those probably should be as a field on the struct, definitely it should be possible to configure
    // those thorough a config file.
    pub const APP_SOCKET_ADDR: &str = "127.0.0.1:5000";

    pub fn new() -> self::Result<Self> {
        Env::load_envs()?;

        Ok(Self)
    }
}

// TODO: Write tests for the Env enum and the .env file 1-1 mapping.
#[cfg(test)]
mod tests {
    use crate::config::Env;
    use std::{
        io::{Read, Seek, Write},
        str::FromStr,
    };

    use anyhow::Context;
    use rand::Rng;
    use std::collections::HashSet;

    // Some API to restore the current-working-directory after the tests that are changingS it.
    #[derive(Debug)]
    struct TempCwd {
        old: std::path::PathBuf,
    }

    impl TempCwd {
        // Changes the current-working-directory to the provided path and returns a TempCwd
        // saves the old cwd to restore it later in the Drop impl.
        fn push<P: AsRef<std::path::Path>>(new: P) -> anyhow::Result<Self> {
            let old = std::env::current_dir().context("Failed to get current dir")?;
            let old = old.clone();

            let new = new.as_ref().to_path_buf().clone();

            std::env::set_current_dir(new.clone()).context("Failed to change cwd")?;

            assert_ne!(old, new);
            let current = std::env::current_dir().context("Failed to get current dir")?;
            assert_eq!(current, new);

            Ok(Self { old })
        }
    }

    // On assertion failure the Drop impl will restore the current-working-directory
    impl Drop for TempCwd {
        fn drop(&mut self) {
            assert_eq!(
                self.old,
                std::path::PathBuf::from("C:\\Dev\\Rust\\rust-web-app")
            );

            println!("Restoring cwd to: {:?}", self.old);
            std::env::set_current_dir(&self.old).unwrap();
        }
    }

    /// Creates a temporary `.env` file in a temporary directory with the provided envs.
    /// Fills the envs with dummy values. Return the set of envs that were written to the file.
    ///
    /// NOTE: Every test that calls this function has to be marked with `#[serial_test::serial]`
    /// since it changes the current-working-directory that is a global state and tests cannot be run in parallel.
    fn create_temp_env_file(vars: &[&str]) -> anyhow::Result<HashSet<String>> {
        let tempdir = tempfile::tempdir()?;
        let path = tempdir.path().join(Env::ENV_PATH);

        let mut file = std::fs::OpenOptions::new()
            .read(true)
            .write(true)
            .truncate(true)
            .create(true)
            .open(&path)
            .context("Failed to open temp env file")?;

        // Store previous cwd and restore it after the test.
        // If not hold into the variable, the Drop will be called immediately and the cwd will be restored before
        let _guard = TempCwd::push(tempdir.path())?;

        // Write every single var to the env file, we do not care about the values.
        for var in vars.iter() {
            writeln!(file, "{}=value", var).context("Failed to write to env file")?;
        }

        let mut buffer = String::new();
        file.seek(std::io::SeekFrom::Start(0))
            .context("Failed to seek to start")?;

        file.read_to_string(&mut buffer)
            .context("Failed to read from env file")?;

        for var in vars.iter() {
            assert!(buffer.contains(var));
        }

        // The idea is that when cwd changes, that would load the .env from the file that was created in that dummy cwd.
        let file_envs = Env::get_file_envs()?;

        Ok(file_envs)
    }

    #[test]
    fn test_enum_variants_are_screaming_case_after_conversion() {
        for env in <Env as strum::IntoEnumIterator>::iter() {
            let as_str = env.to_string();
            assert!(as_str.chars().all(|r| r.is_uppercase() || r == '_'));
        }
    }

    #[test]
    fn check_round_trip_conversion_of_enum_variants() -> anyhow::Result<()> {
        for env in <Env as strum::IntoEnumIterator>::iter() {
            let to_string = env.to_string();

            let to_variant =
                Env::from_str(&to_string).context("Failed to convert back to variant")?;

            assert_eq!(env, to_variant)
        }

        Ok(())
    }

    #[test]
    #[serial_test::serial]
    /// The tests by default run in parallel, on multiple threads. Since we are modifying the current-working-directory
    /// which is a global state, it would cause race conditions.
    ///
    /// If one test changes the cwd and fails to restore it, meaning the Drop kicked in on the TempCwd guard,
    /// the other test would hold in its own TempCwd guard the current-working-directory that was not restored.
    /// Since the envs would not load correctly and the tests would fail.
    ///
    /// We need the serial test, since we are changing the cwd and that is global state.
    /// and it may differ across the tests. It will make the tests run sequential, not parallel.
    ///
    /// Otherwise sometimes the test would not pass, even if it is correct.
    ///
    /// The other way around to make it work would be to run the tests in a single thread:
    /// `cargo test -- --test-threads=1`
    fn test_env_in_file_but_not_in_enum() -> anyhow::Result<()> {
        let vars = vec!["SERVER_URL", "DATABASE_URL"];

        let file_envs = self::create_temp_env_file(vars.as_slice())?;

        // There is not way to emulate the enum variants at runtime, so we just create a HashSet
        // The variants from enum are from iterating the variants using the strum_macros::EnumIter
        // that would be

        // Write one less to enum, to simulate the missing env in the enum.
        let enum_envs =
            HashSet::<String>::from_iter(vars[..vars.len() - 1].iter().map(|s| s.to_string()));

        // Testing for EnvError::MissingEnvFromFile is impossible as we would have to interfere with the enum variants at runtime
        // or we could just test on the already serialized data, but that would be testing the HashSet comparison
        // which feels useless.

        // return Err(EnvError::MissingEnvFromFile(missing_from_file).into());

        assert_ne!(file_envs, enum_envs);

        Ok(())
    }

    #[test]
    #[serial_test::serial]
    /// The tests by default run in parallel, on multiple threads. Since we are modifying the current-working-directory
    /// which is a global state, it would cause race conditions.
    ///
    /// If one test changes the cwd and fails to restore it, meaning the Drop kicked in on the TempCwd guard,
    /// the other test would hold in its own TempCwd guard the current-working-directory that was not restored.
    /// Since the envs would not load correctly and the tests would fail.
    ///
    /// We need the serial test, since we are changing the cwd and that is global state.
    /// and it may differ across the tests. It will make the tests run sequential, not parallel.
    ///
    /// Otherwise sometimes the test would not pass, even if it is correct.
    /// The other way around to make it work would be to run the tests in a single thread:
    /// `cargo test -- --test-threads=1`
    fn test_env_in_enum_but_not_in_file() -> anyhow::Result<()> {
        let vars = vec!["SERVER_URL", "DATABASE_URL"];

        let file_envs = self::create_temp_env_file(vars.as_slice())?;

        // Write one more to enum, to simulate the missing env in the file.
        let enum_envs = HashSet::<String>::from_iter(
            vars.iter()
                .chain(std::iter::once(&"CLIENT_URL"))
                .map(|s| s.to_string()),
        );

        assert_ne!(file_envs, enum_envs);

        Ok(())
    }

    #[test]
    #[serial_test::serial]
    fn test_env_in_file_wrong_format() {
        let vars = vec!["SERVER_URL", "DATABASE_URL", "NotScreamingCase"];

        let result = self::create_temp_env_file(vars.as_slice());

        let err = result.expect_err("Expected error, but got ok");

        let config_error = err
            .downcast_ref::<super::Error>()
            .expect("Expected config::Error");

        assert!(
            matches!(
                config_error,
                crate::config::Error::Env(crate::config::EnvError::WrongFormat(_))
            ),
            "Expected WrongFormat error, but got {:?}",
            config_error
        );
    }

    #[test]
    #[serial_test::serial]
    fn test_env_in_file_duplicated() {
        let vars = vec!["SERVER_URL", "DATABASE_URL", "SERVER_URL"];

        let err =
            self::create_temp_env_file(vars.as_slice()).expect_err("Expected error, but got ok");

        assert!(
            matches!(
                err.downcast_ref::<super::Error>(),
                Some(crate::config::Error::Env(
                    crate::config::EnvError::DuplicatedEnvInFile(_)
                ))
            ),
            "Expected DuplicatedEnvInFile error, but got {:?}",
            err
        );
    }

    #[test]
    #[serial_test::serial]
    fn test_check_missing_env_from_enum() {
        // Generate a key that's almost surely not in the enum
        let random_key: String = rand::rng()
            .sample_iter(&rand::distr::Alphabetic)
            .take(16)
            .map(char::from)
            .collect();

        // Force uppercase to match the SCREAMING_SNAKE_CASE requirement
        let random_key = random_key.to_uppercase();
        assert!(random_key.chars().all(|c| c.is_uppercase() || c == '_'));

        let vars = vec!["SERVER_URL", "DATABASE_URL", &random_key];

        // That fails because it tries to convert the random_key to enum variant
        // with the from_str
        let file_envs = self::create_temp_env_file(&vars).unwrap_err();
        let err = file_envs
            .downcast_ref::<super::Error>()
            .expect("Expected config::Error");

        assert!(matches!(
            err,
            crate::config::Error::Env(crate::config::EnvError::MissingEnvFromEnum(_))
        ));
    }

    #[test]
    #[serial_test::serial]
    fn test_check_missing_env_from_file() {
        let enum_envs = Env::get_enum_envs().unwrap();

        if !enum_envs.is_empty() {
            let mut file_envs = enum_envs.clone().into_iter();

            let first = file_envs.next().unwrap().clone();
            let file_envs = file_envs.collect::<HashSet<String>>();

            let result = Env::check_mapping(enum_envs, file_envs);
            let result = result.unwrap_err();

            assert!(
                matches!(
                    result,
                    crate::config::Error::Env(crate::config::EnvError::MissingEnvFromFile(ref missing))
                    if missing == &HashSet::from_iter(vec![first])
                ),
                "Expected MissingEnvFromFile error with correct missing env, but got {:?}",
                result
            );
        }
    }
}
