use clap::Parser;
use serde_derive::{Deserialize, Serialize};
use std::fs;

#[derive(Parser, Debug)]
pub struct CLIArguments {
    /// path to configuration file
    #[clap(long, value_parser)]
    pub config_path: String,
}

/// [RobotConfig] defines attributes for current RobotConfig
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RobotConfig {
    // name/id of the robot
    pub id: String,
    // path for in-memory artifacts for Robot.
    pub db_path: String,
    // rabbit mq hub password
    pub queue_hub_pw: String,
    // rabbit mq user id
    pub queue_hub_user: String,
    // minimum battery SOC required for operating this robot
    pub lower_soc_limit: f64,
    // time difference in milliseconds between two messages
    pub timeout: u64,
    // rabbit_mq hub hostname
    pub hostname: String,
    // listening port for hub
    pub hub_listening_port: u64,
    // queue name
    pub logs_dir: String,
    // path to init state JSON file
    pub init_state_path: String,
}

/// `load_config` loads collision monitoring configuration into memory.
pub(crate) fn load_config(config_path: &str) -> std::result::Result<RobotConfig, String> {
    match fs::read_to_string(config_path) {
        Ok(file_str) => {
            let ret: RobotConfig = match toml::from_str(&file_str) {
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
