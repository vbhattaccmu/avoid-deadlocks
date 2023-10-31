use crate::collision_monitor::{CollisionMonitor, Robot};
use crate::config::CollisionMonitorConfig;
use amiquip::{
    AmqpProperties, Connection, ConsumerMessage, ConsumerOptions, Exchange, Publish,
    QueueDeclareOptions, Result,
};
use std::sync::Arc;

pub(crate) struct Server;

impl Server {
    /// `start` spins up a Collission Monitor Server
    pub(crate) fn start(config: CollisionMonitorConfig, db: Arc<sled::Db>) -> Result<()> {
        let mut robot_states: Vec<Robot> = Vec::with_capacity(config.num_agents);
        let mut reply_states: Vec<String> = Vec::with_capacity(config.num_agents);
        let mut correlation_ids: Vec<String> = Vec::with_capacity(config.num_agents);

        // open connection.
        let mut connection = Connection::insecure_open(&format!(
            "amqp://{}:{}@{}:{}",
            config.queue_hub_user, config.queue_hub_pw, config.hostname, config.hub_listening_port
        ))?;

        // start collision_monitor.
        let collision_monitor = CollisionMonitor::new(config);

        // open a channel - None says let the library choose the channel ID.
        let channel = connection.open_channel(None)?;

        // get a handle to the default direct exchange.
        let exchange = Exchange::direct(&channel);

        // declare the queue with routing key that will send/receive RPC requests.
        let queue = channel.queue_declare("rpc_queue", QueueDeclareOptions::default())?;

        // start a consumer.
        let consumer = queue.consume(ConsumerOptions::default())?;

        for (_, message) in consumer.receiver().iter().enumerate() {
            match message {
                ConsumerMessage::Delivery(delivery) => {
                    let (reply_to, corr_id) = match (
                        delivery.properties.reply_to(),
                        delivery.properties.correlation_id(),
                    ) {
                        (Some(r), Some(c)) => (r.clone(), c.clone()),
                        _ => {
                            consumer.ack(delivery)?;
                            continue;
                        }
                    };

                    let robot_state: Robot = serde_json::from_slice(&delivery.body)
                        .expect("could not deserialize robot state");

                    robot_states.push(robot_state);
                    reply_states.push(reply_to);
                    correlation_ids.push(corr_id);

                    // now trigger collision monitoring once all states are collected
                    if let Ok(updated_states) =
                        collision_monitor.trigger_collision_monitor(robot_states.clone())
                    {
                        for (idx, state) in updated_states.iter().enumerate() {
                            log::info!(
                                "Sending Updated State to ID {:?}: {:?}",
                                state.device_id,
                                state
                            );
                            // if updated state found, publish it to it own queue.
                            exchange
                                .publish(Publish::with_properties(
                                    serde_json::to_string(&state)
                                        .expect("Could not serialize")
                                        .as_bytes(),
                                    reply_states[idx].clone(),
                                    AmqpProperties::default()
                                        .with_correlation_id(correlation_ids[idx].clone()),
                                ))
                                .expect("Failed to publish message");

                            db.insert(
                                &state.device_id,
                                serde_json::to_string(&state)
                                    .expect("Could not serialize")
                                    .as_bytes()
                                    .to_vec(),
                            )
                            .expect("Failed to insert record");
                        }

                        robot_states.clear();
                        correlation_ids.clear();
                        reply_states.clear();
                    }

                    consumer.ack(delivery)?;
                }
                other => {
                    log::info!("Consumer ended: {:?}", other);
                    break;
                }
            }
        }

        connection.close()
    }
}
