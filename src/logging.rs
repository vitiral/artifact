use std::result;
use log;

use fern;

pub fn init_logger(
    quiet: bool,
    verbosity: u8,
    stderr: bool,
) -> result::Result<(), log::SetLoggerError> {
    let level = if quiet {
        log::LogLevelFilter::Off
    } else {
        match verbosity {
            0 => log::LogLevelFilter::Warn,
            1 => log::LogLevelFilter::Info,
            2 => log::LogLevelFilter::Debug,
            3 => log::LogLevelFilter::Trace,
            _ => unreachable!(),
        }
    };
    let output = if stderr {
        fern::Output::stderr("\n")
    } else {
        fern::Output::stdout("\n")
    };

    fern::Dispatch::new()
        .format(|out, msg, record| {
            out.finish(format_args!("{}: {}", record.level(), msg))
        })
        .level(level)
        .chain(output)
        .apply()
}

#[cfg(test)]
pub fn init_logger_test() {
    match init_logger(false, 3, false) {
        Ok(_) => {}
        Err(_) => {}
    }
}
