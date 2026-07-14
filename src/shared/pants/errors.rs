use thiserror::Error;

#[derive(Error, Debug)]
pub enum PantsCommandError {
    #[error("Error while executing `{command}`")]
    ExecutionError {
        command: String,
        #[source]
        source: std::io::Error,
    },
    #[error("Cannot convert given data `{0}`")]
    ConversionError(#[from] serde_json::Error),
}

pub type PantsResult<T> = core::result::Result<T, PantsCommandError>;
