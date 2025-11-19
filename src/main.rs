use rust_web_app::{config, logger, prelude::*};

#[tokio::main]
async fn main() -> Result<()> {
    let config = match config::Config::new() {
        Ok(config) => config,
        Err(err) => {
            error!(?err, "Error loading configuration");
            return Err(err.into());
        }
    };

    if let Err(err) = logger::init() {
        error!(?err, "Error initializing logger");
        return Err(err);
    }

    if let Err(err) = rust_web_app::run(config).await {
        error!(?err, "Error running application");
        return Err(err);
    }

    return Ok(());
}
