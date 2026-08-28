use thiserror::Error;

#[derive(Error, Debug)]
pub enum PantsCommandError {
    #[error("Error while executing `{command}`")]
    ExecutionError {
        command: String,
        #[source]
        source: std::io::Error,
    },
    #[error("Command `{command}` failed with status {status}: {stderr}")]
    CommandFailed {
        command: String,
        status: String,
        stderr: String,
    },
    #[error("Cannot convert given data `{0}`")]
    ConversionError(#[from] serde_json::Error),
}

pub type PantsResult<T> = core::result::Result<T, PantsCommandError>;
