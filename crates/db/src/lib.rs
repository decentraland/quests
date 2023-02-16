pub mod core;

use std::str::FromStr;

use crate::core::{
    definitions::{
        AddEvent, CreateQuest, Event, QuestInstance, QuestsDatabase, StoredQuest, UpdateQuest,
    },
    errors::{DBError, DBResult},
    ops::{Connect, GetConnection, Migrate},
};
use sqlx::{
    pool::PoolConnection,
    postgres::{PgConnectOptions, PgPoolOptions},
    Error, PgPool, Postgres, Row,
};

use uuid::Uuid;

pub use sqlx::Executor;

pub struct DatabaseOptions {
    url: String,
    pub pool_options: PgPoolOptions,
}

impl DatabaseOptions {
    pub fn new(url: &str) -> Self {
        Self {
            url: url.to_string(),
            pool_options: PgPoolOptions::new(),
        }
    }
}

#[async_trait::async_trait]
impl Connect for DatabaseOptions {
    type Pool = Database;

    async fn connect(self) -> DBResult<Self::Pool> {
        let pg_options = PgConnectOptions::from_str(&self.url).unwrap();
        let pool = self
            .pool_options
            .connect_with(pg_options)
            .await
            .map_err(|_| DBError::UnableToConnect)?;

        Ok(Database::new(pool))
    }
}

#[derive(Clone)]
pub struct Database {
    pub pool: PgPool,
}

impl Database {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl GetConnection for Database {
    type Conn = PoolConnection<Postgres>;

    async fn get_conn(&self) -> DBResult<Self::Conn> {
        if let Ok(conn) = self.pool.acquire().await {
            Ok(conn)
        } else {
            Err(DBError::UnableToConnect)
        }
    }
}

#[async_trait::async_trait]
impl QuestsDatabase for Database {
    async fn ping(&self) -> bool {
        use sqlx::Connection;

        if let Ok(mut con) = self.pool.acquire().await {
            con.ping().await.is_ok()
        } else {
            false
        }
    }

    async fn get_quests(&self, offset: i64, limit: i64) -> DBResult<Vec<StoredQuest>> {
        let query_result = sqlx::query("SELECT * FROM quests OFFSET $1 LIMIT $2")
            .bind(offset)
            .bind(limit)
            .fetch_all(&self.pool)
            .await
            .map_err(|err| DBError::CreateQuestFailed(Box::new(err)))?;

        let mut quests = vec![];

        for row in query_result {
            quests.push(StoredQuest {
                id: parse_uuid_to_str(
                    row.try_get("id")
                        .map_err(|err| DBError::RowCorrupted(Box::new(err)))?,
                ),
                name: row
                    .try_get("name")
                    .map_err(|err| DBError::RowCorrupted(Box::new(err)))?,
                description: row
                    .try_get("description")
                    .map_err(|err| DBError::RowCorrupted(Box::new(err)))?,
                definition: row
                    .try_get("definition")
                    .map_err(|err| DBError::RowCorrupted(Box::new(err)))?,
            })
        }

        Ok(quests)
    }

    async fn create_quest(&self, quest: &CreateQuest) -> DBResult<String> {
        let CreateQuest {
            name,
            description,
            definition,
        } = quest;

        let id = Uuid::new_v4().to_string();
        sqlx::query(
            "INSERT INTO quests (id, name, description, definition) VALUES ($1, $2, $3, $4)",
        )
        .bind(parse_str_to_uuid(&id)?)
        .bind(name)
        .bind(description)
        .bind(definition)
        .execute(&self.pool)
        .await
        .map_err(|err| DBError::CreateQuestFailed(Box::new(err)))?;

        Ok(id)
    }

    async fn update_quest(&self, quest_id: &str, quest: &UpdateQuest) -> DBResult<()> {
        let UpdateQuest {
            name,
            description,
            definition,
        } = quest;
        sqlx::query("UPDATE quests SET name = $1, description = $2, definition = $3, updated_at = $4 WHERE id = $5")
            .bind(name)
            .bind(description)
            .bind(definition)
            .bind(sqlx::types::chrono::Utc::now().naive_utc())
            .bind(parse_str_to_uuid(quest_id)?)
            .execute(&self.pool)
            .await
            .map_err(|err| DBError::UpdateQuestFailed(Box::new(err)))?;

        Ok(())
    }

    async fn get_quest(&self, id: &str) -> DBResult<StoredQuest> {
        let query_result = sqlx::query("SELECT * FROM quests WHERE id = $1")
            .bind(parse_str_to_uuid(id)?)
            .fetch_one(&self.pool)
            .await
            .map_err(|err| match err {
                Error::RowNotFound => DBError::RowNotFound,
                _ => DBError::GetQuestFailed(Box::new(err)),
            })?;

        Ok(StoredQuest {
            id: id.to_string(),
            name: query_result.try_get("name").unwrap(),
            description: query_result.try_get("description").unwrap(),
            definition: query_result.try_get("definition").unwrap(),
        })
    }

    async fn delete_quest(&self, id: &str) -> DBResult<()> {
        sqlx::query("DELETE FROM quests WHERE id = $1")
            .bind(parse_str_to_uuid(id)?)
            .execute(&self.pool)
            .await
            .map_err(|err| DBError::GetQuestFailed(Box::new(err)))?;

        Ok(())
    }

    async fn start_quest(&self, quest_id: &str, user_address: &str) -> DBResult<String> {
        let id = Uuid::new_v4().to_string();

        sqlx::query("INSERT INTO quest_instances (id, quest_id, user_address) VALUES ($1, $2, $3)")
            .bind(parse_str_to_uuid(&id)?)
            .bind(parse_str_to_uuid(quest_id)?)
            .bind(user_address)
            .execute(&self.pool)
            .await
            .map_err(|err| DBError::StartQuestFailed(Box::new(err)))?;

        Ok(id)
    }

    async fn get_quest_instance(&self, id: &str) -> DBResult<QuestInstance> {
        let query_result = sqlx::query("SELECT * FROM quest_instances WHERE id = $1")
            .bind(parse_str_to_uuid(id)?)
            .fetch_one(&self.pool)
            .await
            .map_err(|err| match err {
                Error::RowNotFound => DBError::RowNotFound,
                _ => DBError::GetQuestFailed(Box::new(err)),
            })?;

        // QuestInstance uses a number as the timestamp (unix time) but SQLX returns a specific type (chrono)
        let start_timestamp = date_time_to_unix(
            query_result
                .try_get("start_timestamp")
                .map_err(|err| DBError::RowCorrupted(Box::new(err)))?,
        );

        Ok(QuestInstance {
            id: id.to_string(),
            quest_id: parse_uuid_to_str(
                query_result
                    .try_get("quest_id")
                    .map_err(|err| DBError::RowCorrupted(Box::new(err)))?,
            ),
            user_address: query_result
                .try_get("user_address")
                .map_err(|err| DBError::RowCorrupted(Box::new(err)))?,
            start_timestamp,
        })
    }

    async fn get_user_quest_instances(&self, user_address: &str) -> DBResult<Vec<QuestInstance>> {
        let query_result = sqlx::query("SELECT * FROM quest_instances WHERE user_address = $1")
            .bind(user_address)
            .fetch_all(&self.pool) // it could be replaced by fetch_many that returns a stream
            .await
            .map_err(|err| DBError::GetQuestInstanceFailed(Box::new(err)))?;

        let mut quests = vec![];

        for row in query_result {
            // not using functional methods due to "question mark"
            quests.push(QuestInstance {
                id: parse_uuid_to_str(
                    row.try_get("id")
                        .map_err(|err| DBError::RowCorrupted(Box::new(err)))?,
                ),
                quest_id: parse_uuid_to_str(
                    row.try_get("quest_id")
                        .map_err(|err| DBError::RowCorrupted(Box::new(err)))?,
                ),
                user_address: row
                    .try_get("user_address")
                    .map_err(|err| DBError::RowCorrupted(Box::new(err)))?,
                start_timestamp: date_time_to_unix(
                    row.try_get("start_timestamp")
                        .map_err(|err| DBError::RowCorrupted(Box::new(err)))?,
                ),
            })
        }

        Ok(quests)
    }

    async fn add_event(&self, event: &AddEvent, quest_instance_id: &str) -> DBResult<()> {
        let id = Uuid::new_v4().to_string();
        sqlx::query(
            "INSERT INTO events (id, user_address, event, quest_instance_id) VALUES ($1, $2, $3, $4)",
        )
        .bind(parse_str_to_uuid(&id)?)
        .bind(event.user_address)
        .bind(&event.event)
        .bind(parse_str_to_uuid(quest_instance_id)?)
        .execute(&self.pool)
        .await
        .map_err(|err| DBError::StartQuestFailed(Box::new(err)))?;

        Ok(())
    }

    async fn get_events(&self, quest_instance_id: &str) -> DBResult<Vec<Event>> {
        let query_result = sqlx::query("SELECT * FROM events WHERE quest_instance_id = $1")
            .bind(parse_str_to_uuid(quest_instance_id)?)
            .fetch_all(&self.pool) // it could be replaced by fetch_many that returns a stream
            .await
            .map_err(|err| DBError::GetQuestInstanceFailed(Box::new(err)))?;

        let mut events = vec![];

        for row in query_result {
            // not using functional methods due to "question mark"
            events.push(Event {
                id: parse_uuid_to_str(
                    row.try_get("id")
                        .map_err(|err| DBError::RowCorrupted(Box::new(err)))?,
                ),
                user_address: row
                    .try_get("user_address")
                    .map_err(|err| DBError::RowCorrupted(Box::new(err)))?,
                event: row
                    .try_get("event")
                    .map_err(|err| DBError::RowCorrupted(Box::new(err)))?,
                quest_instance_id: parse_uuid_to_str(
                    row.try_get("quest_instance_id")
                        .map_err(|err| DBError::RowCorrupted(Box::new(err)))?,
                ),
                timestamp: date_time_to_unix(
                    row.try_get("timestamp")
                        .map_err(|err| DBError::RowCorrupted(Box::new(err)))?,
                ),
            })
        }

        Ok(events)
    }
}

#[async_trait::async_trait]
impl Migrate for Database {
    async fn migrate(&self) -> DBResult<()> {
        if let Err(err) = sqlx::migrate!("./migrations").run(&self.pool).await {
            return Err(DBError::MigrationError(Box::new(err)));
        }

        Ok(())
    }
}

fn date_time_to_unix(time: sqlx::types::chrono::NaiveDateTime) -> i64 {
    time.timestamp()
}

pub async fn create_quests_db_component(db_url: &str) -> DBResult<Database> {
    let mut db_options = DatabaseOptions::new(db_url);
    db_options.pool_options = db_options
        .pool_options
        .min_connections(5)
        .max_connections(10);

    let db_pool = db_options.connect().await;

    match db_pool {
        Ok(db) => {
            db.migrate().await?;
            Ok(db)
        }
        Err(error) => {
            panic!("{error:?}");
        }
    }
}

fn parse_str_to_uuid(id: &str) -> DBResult<sqlx::types::Uuid> {
    match sqlx::types::Uuid::parse_str(id) {
        Ok(id) => Ok(id),
        Err(_) => Err(DBError::NotUUID),
    }
}

fn parse_uuid_to_str(uuid: sqlx::types::Uuid) -> String {
    uuid.to_string()
}
