use amiquip::{Connection, Result};
use std::{path::Path, sync::Arc, thread, time::Duration};

use crate::client::{Robot, RobotRpcClient};
use crate::config::RobotConfig;

pub(crate) struct Server;

impl Server {
    pub(crate) fn start(config: RobotConfig, db: Arc<sled::Db>) -> Result<()> {
        // open connection.
        let mut connection = Connection::insecure_open(&format!(
            "amqp://{}:{}@{}:{}",
            config.queue_hub_user, config.queue_hub_pw, config.hostname, config.hub_listening_port
        ))?;

        // open a channel - None says let the library choose the channel ID.
        let channel = connection.open_channel(None)?;

        // instantiate rpc client
        let rpc_client = RobotRpcClient::new(&channel)?;

        // get init state and save it to DB.
        let init_state = Self::read_init_state_from_file(config.init_state_path);
        let mut current_battery_level: f64 = init_state.battery_level;

        db.insert(
            &config.id,
            serde_json::to_string(&init_state)
                .expect("Could not serialize")
                .as_bytes()
                .to_vec(),
        )
        .expect("Failed to insert record");

        // start the messaging loop
        loop {
            let current_state: Robot =
                serde_json::from_slice(&db.get(&config.id).expect("Failed to get record").unwrap())
                    .expect("Could not deserialize");

            if let Ok(robot_state) = rpc_client.publish_current_state(&current_state) {
                if current_battery_level < config.lower_soc_limit {
                    break;
                }
                current_battery_level = robot_state.battery_level;

                db.insert(
                    &config.id,
                    serde_json::to_string(&robot_state)
                        .expect("Could not serialize")
                        .as_bytes()
                        .to_vec(),
                )
                .expect("Failed to insert record");
            } else {
                log::info!("Cannot Broadcast");
                continue;
            }

            // sleep for 10 milliseconds ( 1 Hz )
            // before sending the message again
            thread::sleep(Duration::from_millis(config.timeout));
        }

        connection.close()
    }

    // `read_init_state_from_file` reads current state from JSON file.
    fn read_init_state_from_file(path: String) -> Robot {
        let contents = std::fs::read(&Path::new(&path)).expect("Failed to open file");

        let init_state: Robot =
            serde_json::from_slice(&contents).expect("Failed to deserialize JSON");

        init_state
    }
}
