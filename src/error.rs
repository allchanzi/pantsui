use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppErr {
    #[error("I/O error")]
    Io(#[from] std::io::Error),
    #[error("Terminal error")]
    Terminal,
}

pub type AppResult<T> = Result<T, AppErr>;
