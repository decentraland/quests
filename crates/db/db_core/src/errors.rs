use std::error::Error as StdError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum DBError {}

/// Convenience type alias for grouping driver-specific errors
pub type BoxDynError = Box<dyn StdError + 'static + Send + Sync>;

/// Generic result data structure
pub type DBResult<V> = std::result::Result<V, DBError>;
