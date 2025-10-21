#[derive(Debug)]
pub enum Error {
    Io(std::io::Error),
    Parse(std::num::ParseIntError),
    Custom(String),
    Database(String),
}

// Create a type alias for Result
pub type Result<T> = std::result::Result<T, Error>;

impl From<rusqlite::Error> for Error {
    fn from(err: rusqlite::Error) -> Self {
        Error::Database(err.to_string())
    }
}
