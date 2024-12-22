use std::fmt::Display;
use tracing::subscriber::SetGlobalDefaultError;
use warhorse_protocol::error::Error;

#[derive(Debug)]
pub struct ClientError(pub String);

impl Display for ClientError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<Error> for ClientError {
    fn from(e: Error) -> Self {
        ClientError(e.0)
    }
}

impl From<SetGlobalDefaultError> for ClientError {
    fn from(e: SetGlobalDefaultError) -> Self {
        ClientError(e.to_string())
    }
}

impl From<Box<dyn std::error::Error>> for ClientError {
    fn from(e: Box<dyn std::error::Error>) -> Self {
        ClientError(e.to_string())
    }
}

impl From<String> for ClientError {
    fn from(e: String) -> Self {
        ClientError(e)
    }
}

impl From<&str> for ClientError {
    fn from(e: &str) -> Self {
        ClientError(e.to_string())
    }
}