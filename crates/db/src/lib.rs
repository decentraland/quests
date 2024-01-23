pub mod core;

use crate::core::{
    definitions::{
        AddEvent, CreateQuest, Event, QuestInstance, QuestRewardHook, QuestRewardItem,
        QuestsDatabase, StoredQuest,
    },
    errors::{DBError, DBResult},
    ops::{Connect, GetConnection, Migrate},
};
use futures_util::StreamExt;
pub use sqlx::Executor;
use sqlx::{
    pool::PoolConnection,
    postgres::{PgConnectOptions, PgPoolOptions},
    types::{chrono::NaiveDateTime, Json},
    ConnectOptions, Error, PgPool, Postgres, QueryBuilder, Row, Transaction,
};
use std::{collections::HashMap, str::FromStr};
use uuid::Uuid;

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
        let pg_options = PgConnectOptions::from_str(&self.url)
            .unwrap()
            .disable_statement_logging()
            .clone();
        let pool = self
            .pool_options
            .connect_with(pg_options)
            .await
            .map_err(DBError::UnableToConnect)?;

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
        self.pool.acquire().await.map_err(DBError::UnableToConnect)
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

    async fn get_active_quests(&self, offset: i64, limit: i64) -> DBResult<Vec<StoredQuest>> {
        let query_result = sqlx::query(
            "
                SELECT * FROM quests
                WHERE id NOT IN 
                (SELECT quest_id as id FROM deactivated_quests)
                OFFSET $1 LIMIT $2
            ",
        )
        .bind(offset)
        .bind(limit)
        .fetch_all(&self.pool)
        .await
        .map_err(|err| DBError::GetQuestsFailed(Box::new(err)))?;

        let mut quests = vec![];

        for row in query_result {
            let created_at: NaiveDateTime = row
                .try_get("created_at")
                .map_err(|e| DBError::RowCorrupted(Box::new(e)))?;

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
                creator_address: row
                    .try_get("creator_address")
                    .map_err(|err| DBError::RowCorrupted(Box::new(err)))?,
                image_url: row
                    .try_get("image_url")
                    .map_err(|err| DBError::RowCorrupted(Box::new(err)))?,
                active: true,
                created_at: created_at.timestamp(),
            })
        }

        Ok(quests)
    }

    async fn get_quests_by_creator_id(
        &self,
        creator_address: &str,
        offset: i64,
        limit: i64,
    ) -> DBResult<Vec<StoredQuest>> {
        // Return the quests that was not updated and replaced with a new one
        let query_result = sqlx::query(
            "
                SELECT q.*, (CASE WHEN dq.quest_id IS NULL THEN true ELSE false END) as active 
                FROM quests q
                LEFT JOIN deactivated_quests dq ON q.id = dq.quest_id
                LEFT JOIN quest_updates uq ON q.id = uq.previous_quest_id
                WHERE q.creator_address = $1 AND uq.id IS NULL
                ORDER BY created_at DESC
                OFFSET $2 LIMIT $3
            ",
        )
        .bind(creator_address)
        .bind(offset)
        .bind(limit)
        .fetch_all(&self.pool)
        .await
        .map_err(|err| DBError::GetQuestsFailed(Box::new(err)))?;

        let mut quests = vec![];

        for row in query_result {
            let created_at: NaiveDateTime = row
                .try_get("created_at")
                .map_err(|e| DBError::RowCorrupted(Box::new(e)))?;

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
                creator_address: row
                    .try_get("creator_address")
                    .map_err(|err| DBError::RowCorrupted(Box::new(err)))?,
                image_url: row
                    .try_get("image_url")
                    .map_err(|err| DBError::RowCorrupted(Box::new(err)))?,
                active: row
                    .try_get("active")
                    .map_err(|err| DBError::RowCorrupted(Box::new(err)))?,
                created_at: created_at.timestamp(),
            })
        }

        Ok(quests)
    }

    async fn create_quest(&self, quest: &CreateQuest, creator_address: &str) -> DBResult<String> {
        let quest_id = if let Some(reward) = &quest.reward {
            let mut tx = self
                .pool
                .begin()
                .await
                .map_err(|err| DBError::TransactionBeginFailed(Box::new(err)))?;

            let quest_id = self
                .do_create_quest(quest, creator_address, Some(&mut tx))
                .await?;

            self.do_add_quest_reward_hook(&quest_id, &reward.hook, Some(&mut tx))
                .await?;

            self.do_add_quest_reward_items(&quest_id, &reward.items, Some(&mut tx))
                .await?;

            tx.commit()
                .await
                .map_err(|err| DBError::TransactionFailed(Box::new(err)))?;

            quest_id
        } else {
            self.do_create_quest(quest, creator_address, None).await?
        };

        Ok(quest_id)
    }

    async fn update_quest(
        &self,
        previous_quest_id: &str,
        quest: &CreateQuest,
        creator_address: &str,
    ) -> DBResult<String> {
        let mut transaction = self
            .pool
            .begin()
            .await
            .map_err(|err| DBError::TransactionBeginFailed(Box::new(err)))?;

        let quest_id = self
            .do_create_quest(quest, creator_address, Some(&mut transaction))
            .await?;
        self.do_deactivate_quest(previous_quest_id, Some(&mut transaction))
            .await?;

        if let Some(reward) = &quest.reward {
            self.do_add_quest_reward_hook(&quest_id, &reward.hook, Some(&mut transaction))
                .await?;
            self.do_add_quest_reward_items(&quest_id, &reward.items, Some(&mut transaction))
                .await?;
        }

        let id = Uuid::new_v4().to_string();
        sqlx::query(
            "INSERT INTO quest_updates (id, quest_id, previous_quest_id) VALUES ($1, $2, $3)",
        )
        .bind(parse_str_to_uuid(&id)?)
        .bind(parse_str_to_uuid(&quest_id)?)
        .bind(parse_str_to_uuid(previous_quest_id)?)
        .execute(&mut transaction)
        .await
        .map_err(|err| DBError::UpdateQuestFailed(Box::new(err)))?;

        transaction
            .commit()
            .await
            .map_err(|err| DBError::TransactionFailed(Box::new(err)))?;

        Ok(quest_id)
    }

    async fn get_quest(&self, id: &str) -> DBResult<StoredQuest> {
        let query_result = sqlx::query(
            "
            SELECT q.*, (CASE WHEN dq.quest_id IS NULL THEN true ELSE false END) as active 
            FROM quests q 
            LEFT JOIN deactivated_quests dq ON dq.quest_id = q.id
            WHERE q.id = $1",
        )
        .bind(parse_str_to_uuid(id)?)
        .fetch_one(&self.pool)
        .await
        .map_err(|err| match err {
            Error::RowNotFound => DBError::RowNotFound,
            _ => DBError::GetQuestFailed(Box::new(err)),
        })?;

        let created_at: NaiveDateTime = query_result
            .try_get("created_at")
            .map_err(|e| DBError::RowCorrupted(Box::new(e)))?;

        Ok(StoredQuest {
            id: id.to_string(),
            name: query_result
                .try_get("name")
                .map_err(|e| DBError::RowCorrupted(Box::new(e)))?,
            description: query_result
                .try_get("description")
                .map_err(|e| DBError::RowCorrupted(Box::new(e)))?,
            definition: query_result
                .try_get("definition")
                .map_err(|e| DBError::RowCorrupted(Box::new(e)))?,
            creator_address: query_result
                .try_get("creator_address")
                .map_err(|e| DBError::RowCorrupted(Box::new(e)))?,
            image_url: query_result
                .try_get("image_url")
                .map_err(|e| DBError::RowCorrupted(Box::new(e)))?,
            active: query_result
                .try_get("active")
                .map_err(|err| DBError::RowCorrupted(Box::new(err)))?,
            created_at: created_at.timestamp(),
        })
    }

    async fn is_active_quest(&self, quest_id: &str) -> DBResult<bool> {
        let quest_exists: bool = sqlx::query_scalar(
            "
                SELECT EXISTS (SELECT 1 FROM quests
                WHERE id = $1 AND id NOT IN (SELECT quest_id as id FROM deactivated_quests WHERE quest_id = $1))
            ",
        )
        .bind(parse_str_to_uuid(quest_id)?)
        .fetch_one(&self.pool)
        .await
        .map_err(|err| DBError::GetActiveQuestFailed(Box::new(err)))?;

        Ok(quest_exists)
    }

    async fn abandon_quest_instance(&self, quest_instance_id: &str) -> DBResult<String> {
        let id = Uuid::new_v4().to_string();
        let query = sqlx::query(
            "INSERT INTO abandoned_quest_instances (id, quest_instance_id) VALUES ($1, $2)",
        )
        .bind(parse_str_to_uuid(&id)?)
        .bind(parse_str_to_uuid(quest_instance_id)?);
        query
            .execute(&self.pool)
            .await
            .map_err(|err| DBError::DeactivateQuestFailed(Box::new(err)))
            .map(|_| id)
    }

    async fn complete_quest_instance(&self, quest_instance_id: &str) -> DBResult<String> {
        let id = Uuid::new_v4().to_string();
        let query = sqlx::query(
            "INSERT INTO completed_quest_instances (id, quest_instance_id) VALUES ($1, $2)",
        )
        .bind(parse_str_to_uuid(&id)?)
        .bind(parse_str_to_uuid(quest_instance_id)?);
        query
            .execute(&self.pool)
            .await
            .map_err(|err| DBError::CompleteQuestInstanceFailed(Box::new(err)))
            .map(|_| id)
    }

    async fn is_completed_instance(&self, quest_instance_id: &str) -> DBResult<bool> {
        let quest_instance_exists: bool = sqlx::query_scalar(
            "SELECT EXISTS (SELECT 1 FROM completed_quest_instances WHERE quest_instance_id = $1)",
        )
        .bind(parse_str_to_uuid(quest_instance_id)?)
        .fetch_one(&self.pool)
        .await
        .map_err(|err| DBError::IsCompletedInstanceFailed(Box::new(err)))?;

        Ok(quest_instance_exists)
    }

    async fn is_active_quest_instance(&self, quest_instance_id: &str) -> DBResult<bool> {
        let quest_instance_exists: bool = sqlx::query_scalar(
            "
                SELECT EXISTS (SELECT 1 FROM quest_instances
                WHERE id = $1 AND id NOT IN (SELECT quest_instance_id as id FROM abandoned_quest_instances))
            ",
        )
        .bind(parse_str_to_uuid(quest_instance_id)?)
        .fetch_one(&self.pool)
        .await
        .map_err(|err| DBError::GetActiveQuestInstanceFailed(Box::new(err)))?;

        Ok(quest_instance_exists)
    }

    async fn has_active_quest_instance(
        &self,
        user_address: &str,
        quest_id: &str,
    ) -> DBResult<bool> {
        let quest_instance_exists: bool = sqlx::query_scalar(
            "
                SELECT EXISTS (SELECT 1 FROM quest_instances
                WHERE user_address = $1 AND quest_id = $2 AND id NOT IN (SELECT quest_instance_id as id FROM abandoned_quest_instances))
            ",
        )
        .bind(user_address)
        .bind(parse_str_to_uuid(quest_id)?)
        .fetch_one(&self.pool)
        .await
        .map_err(|err| DBError::HasActiveQuestInstanceFailed(quest_id.to_string(), user_address.to_string(), Box::new(err)))?;

        Ok(quest_instance_exists)
    }

    async fn deactivate_quest(&self, quest_id: &str) -> DBResult<String> {
        self.do_deactivate_quest(quest_id, None).await
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
                _ => DBError::GetQuestInstanceFailed(Box::new(err)),
            })?;

        Ok(QuestInstance::try_from(query_result)?)
    }

    async fn get_active_user_quest_instances(
        &self,
        user_address: &str,
    ) -> DBResult<Vec<QuestInstance>> {
        let query_result = sqlx::query(
            "SELECT * FROM quest_instances 
            WHERE user_address = $1 
            AND id NOT IN (SELECT quest_instance_id as id FROM abandoned_quest_instances)",
        )
        .bind(user_address)
        .fetch_all(&self.pool) // it could be replaced by fetch_many that returns a stream
        .await
        .map_err(|err| {
            DBError::GetActiveQuestInstancesFailed(user_address.to_string(), Box::new(err))
        })?;

        let mut quests = vec![];

        for row in query_result {
            // not using functional methods due to "question mark"
            quests.push(QuestInstance::try_from(row)?)
        }

        Ok(quests)
    }

    async fn add_event(&self, event: &AddEvent, quest_instance_id: &str) -> DBResult<()> {
        sqlx::query(
            "INSERT INTO events (id, user_address, event, quest_instance_id) VALUES ($1, $2, $3, $4)",
        )
        .bind(parse_str_to_uuid(&event.id)?)
        .bind(event.user_address)
        .bind(&event.event)
        .bind(parse_str_to_uuid(quest_instance_id)?)
        .execute(&self.pool)
        .await
        .map_err(|err| DBError::CreateQuestEventFailed(Box::new(err)))?;

        Ok(())
    }

    async fn get_events(&self, quest_instance_id: &str) -> DBResult<Vec<Event>> {
        let query_result =
            sqlx::query("SELECT * FROM events WHERE quest_instance_id = $1 ORDER BY timestamp ASC")
                .bind(parse_str_to_uuid(quest_instance_id)?)
                .fetch_all(&self.pool) // it could be replaced by fetch_many that returns a stream
                .await
                .map_err(|err| DBError::GetQuestEventsFailed(Box::new(err)))?;

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

    async fn add_reward_hook_to_quest(
        &self,
        quest_id: &str,
        reward: &QuestRewardHook,
    ) -> DBResult<()> {
        self.do_add_quest_reward_hook(quest_id, reward, None).await
    }

    async fn get_quest_reward_hook(&self, quest_id: &str) -> DBResult<QuestRewardHook> {
        self.do_get_quest_reward_hook(quest_id, None).await
    }

    async fn add_reward_items_to_quest(
        &self,
        quest_id: &str,
        items: &[QuestRewardItem],
    ) -> DBResult<()> {
        self.do_add_quest_reward_items(quest_id, items, None).await
    }

    async fn get_quest_reward_items(&self, quest_id: &str) -> DBResult<Vec<QuestRewardItem>> {
        self.do_get_quest_reward_items(quest_id, None).await
    }

    async fn get_quest_instances_by_quest_id(
        &self,
        quest_id: &str,
    ) -> DBResult<(Vec<QuestInstance>, Vec<QuestInstance>)> {
        let uuid = parse_str_to_uuid(quest_id)?;
        let instances = sqlx::query(
            "SELECT *, true as active FROM quest_instances 
            WHERE quest_id = $1 
            AND id NOT IN (SELECT quest_instance_id as id FROM abandoned_quest_instances) 

            UNION 
            
            SELECT *, false as active FROM quest_instances 
            WHERE quest_id = $1 
            AND id IN (SELECT quest_instance_id as id FROM abandoned_quest_instances)",
        )
        .bind(uuid)
        .fetch_all(&self.pool) // it could be replaced by fetch_many that returns a stream
        .await
        .map_err(|err| {
            DBError::GetQuestInstancesByQuestIdFailed(quest_id.to_string(), Box::new(err))
        })?;

        let mut actives = vec![];
        let mut not_actives = vec![];

        for instance in instances {
            let active: bool = instance
                .try_get("active")
                .map_err(|err| DBError::RowCorrupted(Box::new(err)))?;

            if active {
                actives.push(QuestInstance::try_from(instance)?);
            } else {
                not_actives.push(QuestInstance::try_from(instance)?);
            }
        }

        Ok((actives, not_actives))
    }

    async fn can_activate_quest(&self, quest_id: &str) -> DBResult<bool> {
        let quest_exists: bool = sqlx::query_scalar(
            "
                SELECT EXISTS (SELECT 1 FROM deactivated_quests
                WHERE quest_id = $1 AND quest_id NOT IN (SELECT previous_quest_id FROM quest_updates where previous_quest_id = $1))
            ",
        )
        .bind(parse_str_to_uuid(quest_id)?)
        .fetch_one(&self.pool)
        .await
        .map_err(|err| DBError::CanActivateQuestFailed(Box::new(err)))?;

        Ok(quest_exists)
    }

    async fn activate_quest(&self, quest_id: &str) -> DBResult<bool> {
        let result = sqlx::query(
            "
                DELETE FROM deactivated_quests WHERE quest_id = $1
            ",
        )
        .bind(parse_str_to_uuid(quest_id)?)
        .execute(&self.pool)
        .await
        .map_err(|err| DBError::ActivateQuestFailed(Box::new(err)))?;

        if result.rows_affected() == 0 {
            Ok(false)
        } else {
            Ok(true)
        }
    }

    async fn is_updatable(&self, quest_id: &str) -> DBResult<bool> {
        let quest_exists: bool = sqlx::query_scalar(
            "
                SELECT EXISTS (SELECT 1 FROM quest_updates
                WHERE previous_quest_id = $1)
            ",
        )
        .bind(parse_str_to_uuid(quest_id)?)
        .fetch_one(&self.pool)
        .await
        .map_err(|err| DBError::IsUpdatableFailed(Box::new(err)))?;

        Ok(!quest_exists)
    }

    async fn get_old_quest_versions(&self, quest_id: &str) -> DBResult<Vec<String>> {
        #[derive(sqlx::FromRow, Debug)]
        struct QuestUpdate {
            quest_id: Uuid,
            previous_quest_id: Uuid,
        }

        let mut quest_updates = sqlx::query_as::<_, QuestUpdate>(
            "SELECT * FROM quest_updates ORDER BY created_at DESC",
        )
        .fetch(&self.pool);

        let mut old_quest_versions = vec![];
        let mut quest_id = quest_id.to_string();
        while let Some(Ok(quest_update)) = quest_updates.next().await {
            if quest_update.quest_id.to_string() == quest_id {
                old_quest_versions.push(quest_update.previous_quest_id.to_string());
                quest_id = quest_update.previous_quest_id.to_string();
            }
        }

        Ok(old_quest_versions)
    }
}

impl Database {
    async fn do_create_quest(
        &self,
        quest: &CreateQuest<'_>,
        creator_address: &str,
        tx: Option<&mut Transaction<'_, Postgres>>,
    ) -> DBResult<String> {
        let quest_id = Uuid::new_v4().to_string();
        let query = sqlx::query(
            "INSERT INTO quests (id, name, description, definition, creator_address, image_url) VALUES ($1, $2, $3, $4, $5, $6)",
        )
        .bind(parse_str_to_uuid(&quest_id)?)
        .bind(quest.name)
        .bind(quest.description)
        .bind(&quest.definition)
        .bind(creator_address)
        .bind(quest.image_url);

        let result = if let Some(tx) = tx {
            query.execute(tx).await
        } else {
            query.execute(&self.pool).await
        };
        result
            .map_err(|err| DBError::CreateQuestFailed(Box::new(err)))
            .map(|_| quest_id)
    }

    async fn do_deactivate_quest(
        &self,
        quest_id: &str,
        tx: Option<&mut Transaction<'_, Postgres>>,
    ) -> DBResult<String> {
        let id = Uuid::new_v4().to_string();
        let query = sqlx::query("INSERT INTO deactivated_quests (id, quest_id) VALUES ($1, $2)")
            .bind(parse_str_to_uuid(&id)?)
            .bind(parse_str_to_uuid(quest_id)?);
        let result = if let Some(tx) = tx {
            query.execute(tx).await
        } else {
            query.execute(&self.pool).await
        };
        result
            .map_err(|err| DBError::DeactivateQuestFailed(Box::new(err)))
            .map(|_| id)
    }

    async fn do_get_quest_reward_hook(
        &self,
        quest_id: &str,
        tx: Option<&mut Transaction<'_, Postgres>>,
    ) -> DBResult<QuestRewardHook> {
        let quest_reward = sqlx::query("SELECT * FROM quest_reward_hooks WHERE quest_id = $1")
            .bind(parse_str_to_uuid(quest_id)?);

        let result = if let Some(tx) = tx {
            quest_reward.fetch_one(tx).await
        } else {
            quest_reward.fetch_one(&self.pool).await
        };

        let result = result.map_err(|err| match err {
            Error::RowNotFound => DBError::RowNotFound,
            _ => DBError::GetQuestRewardFailed(Box::new(err)),
        })?;

        let req_body: Option<Json<HashMap<String, String>>> = result
            .try_get("request_body")
            .map_err(|err| DBError::RowCorrupted(Box::new(err)))?;

        let hook = QuestRewardHook {
            webhook_url: result
                .try_get("webhook_url")
                .map_err(|err| DBError::RowCorrupted(Box::new(err)))?,
            request_body: if let Some(body) = req_body {
                Some(body.0)
            } else {
                None
            },
        };

        Ok(hook)
    }

    async fn do_get_quest_reward_items(
        &self,
        quest_id: &str,
        tx: Option<&mut Transaction<'_, Postgres>>,
    ) -> DBResult<Vec<QuestRewardItem>> {
        let query_result = sqlx::query("SELECT * FROM quest_reward_items WHERE quest_id = $1")
            .bind(parse_str_to_uuid(quest_id)?);

        let result = if let Some(tx) = tx {
            query_result.fetch_all(tx).await
        } else {
            query_result.fetch_all(&self.pool).await
        };

        let result = result.map_err(|err| match err {
            Error::RowNotFound => DBError::RowNotFound,
            _ => DBError::GetQuestRewardFailed(Box::new(err)),
        })?;

        let mut items = vec![];

        for row in result {
            items.push(QuestRewardItem {
                name: row
                    .try_get("reward_name")
                    .map_err(|err| DBError::RowCorrupted(Box::new(err)))?,
                image_link: row
                    .try_get("reward_image")
                    .map_err(|err| DBError::RowCorrupted(Box::new(err)))?,
            })
        }

        Ok(items)
    }

    async fn do_add_quest_reward_hook(
        &self,
        quest_id: &str,
        hook: &QuestRewardHook,
        tx: Option<&mut Transaction<'_, Postgres>>,
    ) -> DBResult<()> {
        let query = sqlx::query(
            "INSERT INTO quest_reward_hooks (quest_id, webhook_url, request_body) VALUES ($1, $2, $3)",
        )
        .bind(parse_str_to_uuid(quest_id)?)
        .bind(&hook.webhook_url)
        .bind(Json(&hook.request_body));

        let result = if let Some(tx) = tx {
            query.execute(tx).await
        } else {
            query.execute(&self.pool).await
        };

        result.map_err(|err| DBError::CreateQuestRewardFailed(Box::new(err)))?;

        Ok(())
    }

    async fn do_add_quest_reward_items(
        &self,
        quest_id: &str,
        items: &[QuestRewardItem],
        tx: Option<&mut Transaction<'_, Postgres>>,
    ) -> DBResult<()> {
        let mut builder = QueryBuilder::new(
            "INSERT INTO quest_reward_items (quest_id, reward_name, reward_image)",
        );

        let quest_id = parse_str_to_uuid(quest_id)?;

        builder.push_values(items, |mut b, item| {
            b.push_bind(quest_id)
                .push_bind(&item.name)
                .push_bind(&item.image_link);
        });

        let query = builder.build();

        let result = if let Some(tx) = tx {
            query.execute(tx).await
        } else {
            query.execute(&self.pool).await
        };

        result.map_err(|err| DBError::CreateQuestRewardFailed(Box::new(err)))?;

        Ok(())
    }
}

#[async_trait::async_trait]
impl Migrate for Database {
    async fn migrate(&self) -> DBResult<()> {
        sqlx::migrate!("./migrations")
            .run(&self.pool)
            .await
            .map_err(|err| DBError::MigrationError(Box::new(err)))
    }
}

pub async fn create_quests_db_component(db_url: &str, run_migrations: bool) -> DBResult<Database> {
    let mut db_options = DatabaseOptions::new(db_url);
    db_options.pool_options = db_options
        .pool_options
        .min_connections(5)
        .max_connections(10);

    let db_pool = db_options.connect().await;

    match db_pool {
        Ok(db) => {
            if run_migrations {
                db.migrate().await?;
            }
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

fn date_time_to_unix(time: sqlx::types::chrono::NaiveDateTime) -> i64 {
    time.timestamp()
}

#[cfg(test)]
mod tests {
    use crate::parse_str_to_uuid;

    #[test]
    fn parse_invalid_uuid_fails() {
        let result = parse_str_to_uuid("not_uuid");
        assert!(result.is_err())
    }
}
