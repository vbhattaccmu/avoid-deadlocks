/// `collision_monitor` defines the collision monitoring system
mod collision_monitor;
/// `config` defines configuration for Collission Monitorng System
mod config;
/// `server` defines the curret RPC server for listening to messages from robots
mod server;

/// `error codes` defines error handling for Agent Info REST API
mod error_codes;

/// `routes` defines handlers for Agent Info REST API
mod routes;

use amiquip::Error;
use clap::Parser;
use humantime::Timestamp;
use std::path::Path;
use std::sync::Arc;
use std::time::SystemTime;
use tokio::task;
use warp::{self, Filter};

use crate::config::CLIArguments;
use crate::server::Server;

#[tokio::main]
async fn main() -> Result<(), Error> {
    ///////////////////////////////
    // 1.Load system configuration.
    ///////////////////////////////

    let cli_args = CLIArguments::parse();

    let config = config::load_config(cli_args.config_path.as_str())
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
    let db_instance_rpc = Arc::clone(&db);
    let db_instance_agent_api = Arc::clone(&db);

    /////////////////////////////////
    // 4.Start Collision Monitor RPC
    /////////////////////////////////
    let server_listening_port = config.listening_port;

    task::spawn(async move { Server::start(config, db_instance_rpc) });

    ////////////////////////
    // 5.Start Warp Threads
    ////////////////////////

    let warp_serve = warp::serve(
        routes::index_route()
            .or(routes::agents(db_instance_agent_api))
            .recover(error_codes::handle_rejection)
            .with(warp::cors().allow_any_origin()),
    );

    let (_, server) =
        warp_serve.bind_with_graceful_shutdown(([0, 0, 0, 0], server_listening_port), async move {
            tokio::signal::ctrl_c()
                .await
                .expect("failed to listen to shutdown signal");
        });

    server.await;

    Ok(())
}
