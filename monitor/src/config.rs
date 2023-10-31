use clap::Parser;
use serde_derive::{Deserialize, Serialize};
use std::fs;

#[derive(Parser, Debug)]
pub struct CLIArguments {
    /// path to configuration file
    #[clap(long, value_parser)]
    pub config_path: String,
}

/// [CollisionMonitorConfig] defines attributes for Collision Monitor
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollisionMonitorConfig {
    // width of the robot container
    pub width: f64,
    // height of the robot container
    pub height: f64,
    // rabbit mq hub password
    pub queue_hub_pw: String,
    // rabbit mq user id
    pub queue_hub_user: String,
    // rabbit_mq hub hostname
    pub hostname: String,
    // listening port for rabbitmq
    pub hub_listening_port: u64,
    // number of robot agents participating in the game
    pub num_agents: usize,
    // logs directory
    pub logs_dir: String,
    // listening port to get information of agents
    pub listening_port: u16,
    // sled db path
    pub db_path: String,
}

/// `load_config` loads collision monitoring configuration into memory.
pub(crate) fn load_config(
    config_path: &str,
) -> std::result::Result<CollisionMonitorConfig, String> {
    match fs::read_to_string(config_path) {
        Ok(file_str) => {
            let ret: CollisionMonitorConfig = match toml::from_str(&file_str) {
                Ok(r) => r,
                Err(_) => return Err(format!("config.toml is not a proper toml file.")),
            };
            return Ok(ret);
        }
        Err(e) => {
            return Err(format!(
                "Error: Config file (config.toml) is not found in the correct directory. 
        Please ensure that the configuration directory: \"{}\" exists. ERROR: {:?}",
                config_path, e
            ))
        }
    };
}
