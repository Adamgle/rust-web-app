use tracing_subscriber::{self};

pub fn init() -> crate::Result<()> {
    // let log = std::fs::File::create("logs/logs.log")?;
    // let stdout_log = tracing_subscriber::fmt::layer().pretty();

    // tracing_subscriber::registry()
    //     .with(
    //         stdout_log
    //             .and_then(tracing_subscriber::fmt::layer().with_writer(std::sync::Arc::new(log))),
    //     )
    //     // .with_max_level(tracing::Level::DEBUG)
    //     // .with_writer(std::sync::Arc::new(log))
    //     // .with_thread_ids(true)
    //     // .with_thread_names(true)
    //     .init();

    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        // .with_writer(stdout)
        // .with_thread_ids(true)
        // .with_thread_names(true)
        .init();

    Ok(())
}

// pub fn to_file() {
//     let stdout_log = tracing_subscriber::fmt::layer().pretty();
//     // A layer that logs events to a file.
//     let file = std::fs::File::create("debug.log");
//     let file = match file {
//         Ok(file) => file,
//         Err(error) => panic!("Error: {:?}", error),
//     };
//     let debug_log = tracing_subscriber::fmt::layer().with_writer(std::sync::Arc::new(file));

//     // A layer that collects metrics using specific events.
//     let metrics_layer = /* ... */ tracing_subscriber::filter::LevelFilter::INFO;

//     tracing_subscriber::registry()
//         .with(
//             stdout_log
//                 // Add an `INFO` filter to the stdout logging layer
//                 .with_filter(filter::LevelFilter::INFO)
//                 // Combine the filtered `stdout_log` layer with the
//                 // `debug_log` layer, producing a new `Layered` layer.
//                 .and_then(debug_log)
//                 // Add a filter to *both* layers that rejects spans and
//                 // events whose targets start with `metrics`.
//                 .with_filter(filter::filter_fn(|metadata| {
//                     !metadata.target().starts_with("metrics")
//                 })),
//         )
//         .with(
//             // Add a filter to the metrics label that *only* enables
//             // events whose targets start with `metrics`.
//             metrics_layer.with_filter(filter::filter_fn(|metadata| {
//                 metadata.target().starts_with("metrics")
//             })),
//         )
//         .init();

//     // This event will *only* be recorded by the metrics layer.
//     tracing::info!(target: "metrics::cool_stuff_count", value = 42);

//     // This event will only be seen by the debug log file layer:
//     tracing::debug!("this is a message, and part of a system of messages");

//     // This event will be seen by both the stdout log layer *and*
//     // the debug log file layer, but not by the metrics layer.
//     tracing::warn!("the message is a warning about danger!");

//     todo!()
// }
