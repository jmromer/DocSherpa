use thiserror::Error;

#[derive(Error, Debug)]
pub enum DocGenError {
    #[error("Failed to parse Python code: {0}")]
    ParsingError(String),
    
    #[error("LLM API error: {0}")]
    LlmApiError(String),
    
    #[error("File I/O error: {0}")]
    FileError(#[from] std::io::Error),
    
    #[error("Configuration error: {0}")]
    ConfigError(String),
    
    #[error("Failed to update file content: {0}")]
    UpdateError(String),
    
    #[error("Unknown error: {0}")]
    Unknown(String),
}

pub type DocGenResult<T> = Result<T, DocGenError>;
