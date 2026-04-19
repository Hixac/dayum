use tracing_subscriber::{fmt, prelude::*, EnvFilter};
use tracing_appender::non_blocking::WorkerGuard;

/// Initializes the tracing subscriber with:
/// - Log level from `RUST_LOG` env var (defaults to "info")
/// - Non‑blocking file output (rotated daily in the `logs/` folder)
/// - Optional console output (enabled by default)
pub fn init_logger(enable_console: bool) -> WorkerGuard {
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info"));

    // File writer (daily rotation)
    let file_appender = tracing_appender::rolling::minutely("logs", "app.log");
    let (file_writer, guard) = tracing_appender::non_blocking(file_appender);
    let file_layer = fmt::layer()
        .with_writer(file_writer)
        .with_target(false)
        .with_ansi(false);   // no ANSI colors in file

    let subscriber = tracing_subscriber::registry().with(filter).with(file_layer);

    if enable_console {
        let console_layer = fmt::layer()
            .with_target(false)
            .with_writer(std::io::stderr)
            .with_ansi(true); // colors on console
        subscriber.with(console_layer).init();
    } else {
        subscriber.init();
    }

    guard // must be held for the lifetime of the program
}
