use std::convert::Infallible;
use warp::{self, http, hyper::StatusCode};

#[derive(Debug)]
pub(crate) enum Error {
    IncorrectInput,
    IncorrectDBRecord,
    DeserializationFailure,
}

impl warp::reject::Reject for Error {}

pub(crate) async fn handle_rejection(
    err: warp::reject::Rejection,
) -> Result<impl warp::Reply, Infallible> {
    let (code, message): (StatusCode, u16) = match err.find() {
        Some(Error::IncorrectInput) => (StatusCode::BAD_REQUEST, INCORRECT_INPUT),
        Some(Error::IncorrectDBRecord) => (StatusCode::BAD_REQUEST, INCORRECT_DB_RECORD),
        Some(Error::DeserializationFailure) => (StatusCode::BAD_REQUEST, DESERIALIZATION_FAILURE),
        None => (StatusCode::BAD_REQUEST, DESERIALIZATION_FAILURE),
    };

    Ok(http::Response::builder()
        .status(code)
        .body(message.to_string()))
}

const INCORRECT_INPUT: u16 = 0x835;
const INCORRECT_DB_RECORD: u16 = 0x836;
const DESERIALIZATION_FAILURE: u16 = 0x837;
