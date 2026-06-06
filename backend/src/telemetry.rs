//! Structured logging / tracing initialization.

use tracing_subscriber::{fmt, prelude::*, EnvFilter};

/// Initialize the global tracing subscriber.
///
/// The filter is read from `RUST_LOG` and defaults to `info`. Safe to call
/// once; subsequent calls are ignored.
pub fn init() {
    let filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));

    let _ = tracing_subscriber::registry()
        .with(filter)
        .with(fmt::layer())
        .try_init();
}
