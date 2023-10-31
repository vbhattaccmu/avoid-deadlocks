use amiquip::{
    AmqpProperties, Channel, Consumer, ConsumerMessage, ConsumerOptions, Exchange, Publish, Queue,
    QueueDeclareOptions, Result,
};
use serde_derive::{Deserialize, Serialize};
use uuid::Uuid;

/// [RobotRpcClient] defines current RPC client for sending/receiving to/from the server.
pub struct RobotRpcClient<'a> {
    queue: Queue<'a>,
    consumer: Consumer<'a>,
    exchange: Exchange<'a>,
}

impl<'a> RobotRpcClient<'a> {
    // `new` creates a new client
    pub fn new(channel: &Channel) -> Result<RobotRpcClient> {
        let exchange = Exchange::direct(&channel);

        let queue = channel.queue_declare(
            "",
            QueueDeclareOptions {
                exclusive: true,
                ..QueueDeclareOptions::default()
            },
        )?;
        let consumer = queue.consume(ConsumerOptions {
            no_ack: true,
            ..ConsumerOptions::default()
        })?;

        Ok(RobotRpcClient {
            exchange,
            queue,
            consumer,
        })
    }

    // `publish_current_state` publishes its current state to the server
    // after reply is received it updates its current state on k-v store
    pub fn publish_current_state(&self, robot_state: &Robot) -> Result<Robot> {
        let correlation_id = format!("{}", Uuid::new_v4());

        self.exchange.publish(Publish::with_properties(
            serde_json::to_string(&robot_state)
                .expect("Could not deserialize")
                .as_bytes(),
            "rpc_queue",
            AmqpProperties::default()
                .with_reply_to(self.queue.name().to_string())
                .with_correlation_id(correlation_id.to_string()),
        ))?;

        for message in self.consumer.receiver().iter() {
            match message {
                ConsumerMessage::Delivery(delivery) => {
                    if delivery.properties.correlation_id().as_ref() == Some(&correlation_id) {
                        let updated_robot_state: Robot =
                            serde_json::from_slice(&delivery.body).expect("Could not deserialize");

                        if updated_robot_state.device_id == robot_state.device_id {
                            log::info!("Received data from Hub {:?}", updated_robot_state);
                            return Ok(updated_robot_state);
                        } else {
                            continue;
                        }
                    }
                }
                _ => {
                    break;
                }
            }
        }

        Ok(robot_state.clone())
    }
}

/// [Robot] defines attributes which define the
/// current state of each robot.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Robot {
    /// x-coordinate of the robot
    pub x: f64,
    /// y-coordinate of the robot
    pub y: f64,
    /// angle of inclination to y-axis in radians
    pub theta: f64,
    /// loading status of the robot: true | false
    pub loaded: bool,
    /// current timestamp of the robot
    pub timestamp: i64,
    /// path of the robot
    pub path: Vec<Path>,
    /// device id of the robot
    pub device_id: String,
    /// state of the robot: resume | pending
    pub state: String,
    /// current battery level of the robot
    pub battery_level: f64,
}

/// [Path] defines attributes which define a
/// location of the robot.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Path {
    /// x-coordinate of the robot
    pub x: f64,
    /// y-coordinate of the robot
    pub y: f64,
    /// angle of inclination to y-axis in radians
    pub theta: f64,
}
