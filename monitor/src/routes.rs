use warp::{self, http, Filter};

use std::{convert::Infallible, sync::Arc};

use crate::collision_monitor::Robot;
use crate::error_codes::Error as CollisionMonitorError;

pub(crate) fn index_route(
) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    async fn index_page_handler() -> Result<impl warp::Reply, Infallible> {
        let body = "Collision Monitor".to_string();
        Ok(http::Response::builder().body(body))
    }

    warp::path!().and(warp::get()).and_then(index_page_handler)
}

pub(crate) fn agents(
    db: Arc<sled::Db>,
) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    async fn get_agent_info(
        db: Arc<sled::Db>,
        agent_identidier: String,
    ) -> Result<impl warp::Reply, warp::Rejection> {
        if agent_identidier == String::new() {
            return Err(warp::reject::custom(CollisionMonitorError::IncorrectInput));
        }

        let db_record = match db.get(&agent_identidier).expect("Failed to get record") {
            Some(state) => state,
            None => {
                return Err(warp::reject::custom(
                    CollisionMonitorError::IncorrectDBRecord,
                ));
            }
        };

        let current_state: Robot =
            serde_json::from_slice(&db_record).expect("Could not deserialize record");

        let body = match serde_json::to_string(&current_state) {
            Ok(str) => str,
            Err(_) => {
                return Err(warp::reject::custom(
                    CollisionMonitorError::DeserializationFailure,
                ));
            }
        }
        .as_bytes()
        .to_vec();

        Ok(http::Response::builder()
            .status(http::StatusCode::OK)
            .body(body))
    }

    let agents_route = |db: Arc<sled::Db>| {
        warp::path!("state" / String)
            .and(warp::get())
            .and(warp::path::end())
            .and_then(move |agent| get_agent_info(Arc::clone(&db), agent))
    };

    agents_route(db)
}
