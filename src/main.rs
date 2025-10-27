use rust_web_app::{config, logger, prelude::*};

#[tokio::main]
async fn main() -> Result<()> {
    // TODO: Why do I have to match every single branch here, we need to do something about it stat.

    let config = match config::Config::new() {
        Ok(config) => config,
        Err(message) => {
            error!("Error initializing configuration: {}", message);
            return Err(message.into());
        }
    };

    if let Err(message) = logger::init() {
        error!("Error initializing logger: {}", message);

        return Err(message);
    }

    if let Err(message) = rust_web_app::run(config).await {
        error!("Error running application: {}", message);

        return Err(message);
    }

    return Ok(());
}
