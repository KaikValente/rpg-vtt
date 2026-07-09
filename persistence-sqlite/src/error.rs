use std::fmt;

#[derive(Debug)]
pub enum PersistenceError {
    Sqlite(rusqlite::Error),
    InvalidOperation(String),
    InvalidDuration(String),
    InvalidStacking(String),
    MissingDurationRounds,
}

impl fmt::Display for PersistenceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PersistenceError::Sqlite(e) => write!(f, "SQLite persistence error: {e}"),
            PersistenceError::InvalidOperation(value) => {
                write!(f, "invalid stored effect operation: '{value}'")
            }
            PersistenceError::InvalidDuration(value) => {
                write!(f, "invalid stored effect duration: '{value}'")
            }
            PersistenceError::InvalidStacking(value) => {
                write!(f, "invalid stored effect stacking mode: '{value}'")
            }
            PersistenceError::MissingDurationRounds => {
                write!(f, "stored duration 'rounds' is missing its round count")
            }
        }
    }
}

impl std::error::Error for PersistenceError {}

impl From<rusqlite::Error> for PersistenceError {
    fn from(error: rusqlite::Error) -> Self {
        PersistenceError::Sqlite(error)
    }
}
