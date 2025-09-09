use rust_web_app::{config, logger, prelude::*};

#[tokio::main]
async fn main() -> Result<()> {
    // TODO: Why do I have to match every single branch here, we need to do something about it.
    if let Err(message) = logger::init() {
        eprintln!("Error initializing logger: {}", message);

        return Err(message);
    }

    if let Err(message) = config::Config::new() {
        eprintln!("Error initializing configuration: {}", message);

        return Err(message.into());
    };

    if let Err(message) = rust_web_app::run().await {
        eprintln!("Error running application: {}", message);

        return Err(message.into());
    }

    return Ok(());
}
