#[macro_use] extern crate tracing;

use clap::Parser;
use cli::Cli;

use tracing::Level;
use tracing_subscriber::fmt;

mod cli;

fn main() {
    let cli = Cli::parse();

    let format = fmt::format()
        .with_level(false)
        .with_target(false)
        .with_thread_names(true)
        .compact();

    let level = match cli.debug {
        0 => Level::WARN,
        1 => Level::INFO,
        2 => Level::DEBUG,
        _ => Level::TRACE,
    };

    tracing_subscriber::fmt()
        .with_max_level(level)
        .event_format(format)
        .init();
}
