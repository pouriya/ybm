mod otp;
mod settings;
mod tui;
use std::io;

use anyhow::Context;
use tracing::debug;
use tracing_appender::non_blocking;
use tracing_subscriber::{filter, fmt, util::SubscriberInitExt};

fn main() -> anyhow::Result<()> {
    let tmp_parsed_args = settings::parse();
    let (mut log_level, filename, line_number, targer) = match (
        tmp_parsed_args.trace,
        tmp_parsed_args.debug,
        tmp_parsed_args.quiet,
    ) {
        (true, _, _) => (filter::LevelFilter::TRACE, true, true, true),
        (_, true, _) => (filter::LevelFilter::DEBUG, false, false, true),
        (_, _, true) => (filter::LevelFilter::OFF, false, false, false),
        _ => (filter::LevelFilter::INFO, false, false, false),
    };
    let mut color = true;
    let (writer, _writer_guard) = if let settings::CliSubCommand::Tui { .. } =
        tmp_parsed_args.maybe_subcommand.unwrap_or_default()
    {
        if log_level == filter::LevelFilter::TRACE || log_level == filter::LevelFilter::DEBUG {
            color = false;
            let log_directory =
                std::env::current_dir().context("Could not detect current directory")?;
            tracing_appender::non_blocking(tracing_appender::rolling::never(
                log_directory,
                "ybm.log",
            ))
        } else {
            log_level = filter::LevelFilter::OFF;
            non_blocking(io::stderr())
        }
    } else {
        non_blocking(io::stderr())
    };
    fmt()
        .compact()
        .with_ansi(color)
        .with_level(true)
        .with_file(filename)
        .with_line_number(line_number)
        .with_target(targer)
        .with_writer(writer)
        .with_max_level(log_level)
        .init();

    debug!("started");
    match settings::parse().maybe_subcommand.unwrap_or_default() {
        settings::CliSubCommand::Tui { config_file } => {
            tui::run(&mut crate::settings::ensure_config(config_file)?)?;
        }
        settings::CliSubCommand::From(from) => match from {
            settings::CliFrom::Webcam { device_index } => {
                crate::otp::qrcode::detect_from_webcam(device_index)?;
            }
        },
    }
    Ok(())
}
