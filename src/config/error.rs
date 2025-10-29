use std::collections::HashSet;
use std::sync::Arc;

// We are accessing the Result alias on the mod.rs, the parent module, and convention will be that
// each module will have it's own result to mitigate the redundant calls when converting errors from one module to another.
// NOTE: Think about making file called prelude.rs for each module that will re-export the Result alias and the Error enum
// and bring it to the current scope.
// use super::Result;

#[derive(thiserror::Error, Debug, Clone)]
pub enum Error {
    // TODO: Possibly the naming of the variant could be better, we care about readability, not the type of the error, but since
    // we don't care about defining every possible error of the dotenv crate, it might be fine to just wrap the inner error of the crate
    // in a variant, and then we can just infer from the error what happen.
    #[error(transparent)]
    Env(#[from] EnvError),
    // #[error("Environment variable has wrong format, should be SCREAMING_SNAKE_CASE: {0}")]
    // EnvWrongFormat(#[from] strum::ParseError),
    #[error(transparent)]
    Other(#[from] Arc<anyhow::Error>),
}

#[derive(thiserror::Error, Debug, Clone)]
pub enum EnvError {
    #[error("Error loading .env")]
    Error(#[from] Arc<dotenvy::Error>),
    // #[error("Error iterating over environment variables")]
    // EnvIterator(#[from] dotenvy::Error),
    #[error("Missing environment variable: {0}")]
    MissingEnv(Arc<dotenvy::Error>),
    #[error(
        "Environment variable declared in the .env file has wrong format, should be SCREAMING_SNAKE_CASE: {0}"
    )]
    WrongFormat(String),
    // NOTE: That would not make sense since we have to convert the env to enum variant
    // and if it fails, we don't have the variant to return, and have to abort early.
    // #[error("Environment variable defined in the .env file was not found in the enum: {0:#?}")]
    // MissingEnvFromEnum(HashSet<String>),
    #[error("Environment variable defined in the .env file was not found in the enum: {0}")]
    MissingEnvFromEnum(String),
    // NOTE: That is the case of env declared as variant in the enum, but not present in the .env file.
    // That prevents you from creating variant that do not point to the env declared in the env.
    // That is not even a real error, may save you some stack space, but we want to be sure that the .env file and the enum are in sync.
    // So to make sure that we are not accessing envs that are not defined in the .env file.
    #[error("Environment variable defined in the enum was not found in the .env file: {0:#?}")]
    MissingEnvFromFile(HashSet<String>),
    #[error("Environment variable defined multiple times in the .env file: {0}")]
    DuplicatedEnvInFile(String),
    // NOTE: That could happen if strum_macros will serialize two technically distinct variants to the same SCREAMING_SNAKE_CASE name.
    // Example: database_url and DatabaseUrl, both would serialize to DATABASE_URL when converting to_string.
    #[error(
        "Environment variable defined multiple times in the enum: variant: {variant:?}, translation: {translation}"
    )]
    DuplicatedEnvInEnum {
        variant: super::Env,
        translation: String,
    },
    #[error("I/O error occurred: {0}")]
    Io(#[from] Arc<std::io::Error>),
    #[error("Catch all variant: {0}")]
    Other(#[from] Arc<anyhow::Error>),
}

impl From<dotenvy::Error> for EnvError {
    fn from(err: dotenvy::Error) -> Self {
        EnvError::Error(Arc::new(err))
    }
}

impl From<std::io::Error> for EnvError {
    fn from(err: std::io::Error) -> Self {
        EnvError::Io(Arc::new(err))
    }
}
