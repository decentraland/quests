use sqlx::Error;
use std::error::Error as StdError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum DBError {
    #[error("Unable to connect to DB")]
    UnableToConnect(Error),

    #[error("Unable to migrate: {0}")]
    MigrationError(BoxDynError),

    #[error("Unable to begin transaction: {0}")]
    TransactionBeginFailed(BoxDynError),

    #[error("Unable to commit or rollback transaction: {0}")]
    TransactionFailed(BoxDynError),

    #[error("Unable to create a quest: {0}")]
    CreateQuestFailed(BoxDynError),

    #[error("Unable to get a quest: {0}")]
    GetQuestFailed(BoxDynError),

    #[error("Unable to get a quests")]
    GetQuestsFailed(BoxDynError),

    #[error("Unable to update a quest: {0}")]
    UpdateQuestFailed(BoxDynError),

    #[error("Unable to deactivate a quest: {0}")]
    DeactivateQuestFailed(BoxDynError),

    #[error("Unable to abandon a quest: {0}")]
    AbandonQuestFailed(BoxDynError),

    #[error("Unable to create a quest instance: {0}")]
    StartQuestFailed(BoxDynError),

    #[error("Unable to get a quest instance: {0}")]
    GetQuestInstanceFailed(BoxDynError),

    #[error("Unable to get an event for a quest: {0}")]
    GetQuestEventsFailed(BoxDynError),

    #[error("Unable to store an event for a quest: {0}")]
    CreateQuestEventFailed(BoxDynError),

    #[error("Row has incorrect data: {0}")]
    RowCorrupted(BoxDynError),

    #[error("Not a UUID given")]
    NotUUID,

    #[error("Not found")]
    RowNotFound,
}

/// Convenience type alias for grouping driver-specific errors
pub type BoxDynError = Box<dyn StdError + 'static + Send + Sync>;

/// Generic result data structure
pub type DBResult<V> = Result<V, DBError>;
