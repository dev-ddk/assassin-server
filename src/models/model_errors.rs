use thiserror::Error;
use color_eyre::Report;
use tracing::info;

#[derive(Debug, Error)]
pub enum ModelError {
    #[error("The player is already in an active game")]
    AlreadyInGame,
    #[error("A database error occurred")]
    DatabaseError,
    #[error("The player is currently not in the requested game")]
    NotInGame,
    #[error("Game hasn't started yet")]
    GameNotStarted,
    #[error("The player doesn't currently have a target")]
    NoCurrentTarget,
    #[error("Unknown error")]
    UnknownError(Report)
}

pub type Result<T> = std::result::Result<T, ModelError>;


impl ModelError {
    pub fn error_code(&self) -> String {
        match self {
            Self::AlreadyInGame => "ALREADY_IN_GAME".to_string(),
            Self::DatabaseError => "DATABASE_ERROR".to_string(),
            Self::NotInGame => "NOT_IN_GAME".to_string(),
            Self::GameNotStarted => "GAME_NOT_STARTED".to_string(),
            Self::NoCurrentTarget => "NO_CURRENT_TARGET".to_string(),
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
