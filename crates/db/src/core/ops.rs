use async_trait::async_trait;

use super::{definitions::QuestsDatabase, errors::DBResult};

// TODO: Recheck if it's actually needed
/// Database operations trait(migrations, pool creation and fetching connection from pool)
pub trait DBOps: GetConnection + Migrate {}

// TODO: Recheck if it's actually needed
/// Get database connection
#[async_trait]
pub trait GetConnection {
    /// database connection type
    type Conn;
    /// database specific error-type
    /// get connection from connection pool
    async fn get_conn(&self) -> DBResult<Self::Conn>;
}

/// Create database connection
#[async_trait]
pub trait Connect {
    /// database specific pool-type
    type Pool: QuestsDatabase;
    /// database specific error-type
    /// create connection pool
    async fn connect(self) -> DBResult<Self::Pool>;
}

/// database migrations
#[async_trait]
pub trait Migrate: QuestsDatabase {
    /// database specific error-type
    /// run migrations
    async fn migrate(&self) -> DBResult<()>;
}
