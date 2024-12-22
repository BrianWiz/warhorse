use std::fmt::Display;
use socketioxide::{BroadcastError, SendError};
use warhorse_protocol::error::Error;

#[derive(Debug)]
pub struct ServerError(pub String);

impl Display for ServerError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<Error> for ServerError {
    fn from(e: Error) -> Self {
        ServerError(e.0)
    }
}

impl From<SendError> for ServerError {
    fn from(e: SendError) -> Self {
        ServerError(e.to_string())
    }
}

impl From<BroadcastError> for ServerError {
    fn from(e: BroadcastError) -> Self {
        ServerError(e.to_string())
    }
}

impl From<Box<dyn std::error::Error>> for ServerError {
    fn from(e: Box<dyn std::error::Error>) -> Self {
        ServerError(e.to_string())
    }
}

impl From<String> for ServerError {
    fn from(e: String) -> Self {
        ServerError(e)
    }
}

impl From<&str> for ServerError {
    fn from(e: &str) -> Self {
        ServerError(e.to_string())
    }
}