use thiserror::Error;
use color_eyre::Report;
use tracing::info;

#[derive(Debug, Error)]
pub enum ModelError {
    #[error("The player is already in another active game")]
    AlreadyInAnotherGame,
    #[error("The player is already in the requested game")]
    AlreadyInRequestedGame,
    #[error("A database error occurred")]
    DatabaseError,
    #[error("The player is currently not in the requested game")]
    NotInGame,
    #[error("Game hasn't started yet")]
    GameNotStarted,
    #[error("Game not found")]
    GameNotFound,
    #[error("The player doesn't currently have a target")]
    NoCurrentTarget,
    #[error("User is already registered")]
    AlreadyRegistered,
    #[error("You are not registered yet")]
    NotRegistered,
    #[error("Unknown error")]
    UnknownError(Report)
}

pub type Result<T> = std::result::Result<T, ModelError>;


impl ModelError {
    pub fn error_code(&self) -> String {
        match self {
            Self::AlreadyInAnotherGame => "ALREADY_IN_ANOTHER_GAME".to_string(),
            Self::AlreadyInRequestedGame => "ALREADY_IN_REQUESTED_GAME".to_string(),
            Self::DatabaseError => "DATABASE_ERROR".to_string(),
            Self::NotInGame => "NOT_IN_GAME".to_string(),
            Self::GameNotStarted => "GAME_NOT_STARTED".to_string(),
            Self::GameNotFound => "GAME_NOT_FOUND".to_string(),
            Self::NoCurrentTarget => "NO_CURRENT_TARGET".to_string(),
            Self::AlreadyRegistered => "ALREADY_REGISTERED".to_string(),
            Self::NotRegistered => "NOT_REGISTERED".to_string(),
            Self::UnknownError(_) => "UNKNOWN".to_string()
        }
    }
}

impl From<diesel::result::Error> for ModelError {
    fn from(err: diesel::result::Error) -> Self {
        info!("Database errored: {:?}", err);
        Self::DatabaseError
    }
}

impl From<r2d2::Error> for ModelError {
    fn from(err: r2d2::Error) -> Self {
        info!("Database errored: {:?}", err);
        Self::DatabaseError
    }
}

