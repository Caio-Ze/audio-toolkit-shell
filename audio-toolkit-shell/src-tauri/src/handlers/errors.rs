use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Error, Debug, Serialize, Deserialize)]
pub enum CommandError {
    #[error("Authentication failed: {message}")]
    AuthenticationError { message: String },
    
    #[error("Process error: {message}")]
    ProcessError { message: String },
    
    #[error("File handling error: {message}")]
    FileError { message: String },
    
    #[error("Network error: {message}")]
    NetworkError { message: String },
    
    #[error("Validation error: {message}")]
    ValidationError { message: String },
    
    #[error("Internal error: {message}")]
    InternalError { message: String },
}

impl CommandError {
    pub fn authentication(message: impl Into<String>) -> Self {
        Self::AuthenticationError {
            message: message.into(),
        }
    }
    
    pub fn process(message: impl Into<String>) -> Self {
        Self::ProcessError {
            message: message.into(),
        }
    }
    
    pub fn file(message: impl Into<String>) -> Self {
        Self::FileError {
            message: message.into(),
        }
    }
    
    pub fn network(message: impl Into<String>) -> Self {
        Self::NetworkError {
            message: message.into(),
        }
    }
    
    pub fn validation(message: impl Into<String>) -> Self {
        Self::ValidationError {
            message: message.into(),
        }
    }
    
    pub fn internal(message: impl Into<String>) -> Self {
        Self::InternalError {
            message: message.into(),
        }
    }
}

impl From<anyhow::Error> for CommandError {
    fn from(error: anyhow::Error) -> Self {
        Self::InternalError {
            message: error.to_string(),
        }
    }
}

impl From<CommandError> for String {
    fn from(error: CommandError) -> Self {
        error.to_string()
    }
}

pub type CommandResult<T> = Result<T, CommandError>;