mod client;
mod config;
mod server;

use amiquip::Error;
use clap::Parser;
use humantime::Timestamp;
use std::path::Path;
use std::sync::Arc;
use std::time::SystemTime;

use crate::config::{load_config, CLIArguments};
use crate::server::Server;

fn main() -> Result<(), Error> {
    ///////////////////////////////
    // 1.Load system configuration.
    ///////////////////////////////

    let cli_args = CLIArguments::parse();

    let config = load_config(cli_args.config_path.as_str())
        .expect("Irrecoverable error: failed to load config.toml");

    ///////////////////
    // 2.Set up logger.
    ///////////////////

    std::fs::create_dir_all(&config.logs_dir)
        .expect("Irrecoverable error: failed to create logs directory");
    let proc_start_time = Timestamp::from(SystemTime::now());

    fern::Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!(
                "{}[{}][{}] {}",
                chrono::Local::now().format("[%H:%M:%S]"),
                record.target(),
                record.level(),
                message
            ))
        })
        .level(log::LevelFilter::Info)
        .chain(
            fern::log_file(format!("{}/{}.log", &config.logs_dir, proc_start_time))
                .expect("could not chain logs directory"),
        )
        .apply()
        .expect("could not set up logger");

    ///////////////////
    // 3. Open Sled DB.
    ///////////////////

    let db = Arc::new(sled::open(Path::new(&config.db_path)).expect("Failed to open sled db"));

    //////////////////
    // 4.Start server.
    //////////////////

    Server::start(config, db)
}
