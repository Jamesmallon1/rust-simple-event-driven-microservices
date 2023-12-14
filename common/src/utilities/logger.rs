use chrono::Local;
use colored::*;
use fern::Dispatch;
use log::{error, info, LevelFilter};
use std::io;
use std::string::ToString;

const ERROR_LEVEL: &str = "ERROR";
const WARN_LEVEL: &str = "WARN";
const INFO_LEVEL: &str = "INFO";
const DEBUG_LEVEL: &str = "DEBUG";
const TRACE_LEVEL: &str = "TRACE";

/// Initializes the logging macros for the entire application. You can configure the logging level
/// directly here within the code.
///
/// # Arguments
///
/// * `log_output_file` - The path to the logging file
/// * `microservice_name` - The name of the microservice you are initializing the logger for
pub fn initialize(log_output_file: &str, microservice_name: &str) {
    let verbose = false;
    match configure_logger(verbose, log_output_file) {
        Ok(()) => {
            info!("{} microservice started", microservice_name);
            info!("Logger successfully configured");
        }
        Err(e) => {
            error!("Failed to configure logger: {}", e);
            std::process::exit(1);
        }
    }
}

fn configure_logger(verbose: bool, log_output_file: &str) -> Result<(), fern::InitError> {
    let mut verbosity = LevelFilter::Info;
    if verbose {
        verbosity = LevelFilter::Debug;
    }

    // configure a logger for the console to include the ANSI color codes
    let console_dispatch = Dispatch::new()
        // format: specify log line format
        .format(|out, message, record| {
            let level_string = match record.level() {
                log::Level::Error => ERROR_LEVEL.red(),
                log::Level::Warn => WARN_LEVEL.yellow(),
                log::Level::Info => INFO_LEVEL.blue(),
                log::Level::Debug => DEBUG_LEVEL.into(),
                log::Level::Trace => TRACE_LEVEL.cyan(),
            };
            out.finish(format_args!(
                "{} [{}] [{}] - {}",
                Local::now().format("[%Y-%m-%d][%H:%M:%S]").to_string().blue(),
                record.target().to_uppercase().green(),
                level_string,
                message
            ))
        })
        // output: specify log output
        .chain(io::stdout()) // log to stdout
        .level(verbosity);

    // configure a logger for the file to exclude ANSI color codes
    let file_dispatch = Dispatch::new()
        // format: specify log line format
        .format(|out, message, record| {
            out.finish(format_args!(
                "{} [{}] [{}] - {}",
                chrono::Local::now().format("[%Y-%m-%d][%H:%M:%S]").to_string(),
                record.target().to_uppercase(),
                record.level(),
                message
            ))
        })
        // output: specify log output
        .chain(fern::log_file(log_output_file)?) // log to a file
        .level(verbosity);

    // implement both loggers on the base dispatch logger
    Dispatch::new().chain(file_dispatch).chain(console_dispatch).apply()?;

    Ok(())
}
