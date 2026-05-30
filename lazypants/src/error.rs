#[derive(Debug)]
pub enum AppErr {
    Io,
    Terminal
}


pub type AppResult<T> = Result<T, AppErr>;
