use deadpool_redis::{
    redis::{cmd, RedisResult},
    Connection, CreatePoolError, Pool, Runtime,
};

pub struct Redis {
    pub pool: Pool,
}

impl Redis {
    pub async fn new(redis_url: &str) -> Result<Self, CreatePoolError> {
        let url = format!("redis://{redis_url}");
        log::debug!("Redis URL: {}", url);

        let pool = deadpool_redis::Config::from_url(url).create_pool(Some(Runtime::Tokio1))?;
        let conn = pool.get().await;

        if let Err(err) = conn {
            log::error!("Error on connecting to redis: {:?}", err);
            panic!("Unable to connect to redis {err:?}")
        }
        Ok(Redis { pool })
    }

    pub fn stop(&self) {
        self.pool.close()
    }

    pub async fn get_async_connection(&self) -> Option<Connection> {
        match self.pool.get().await {
            Ok(connection) => Some(connection),
            Err(err) => {
                log::error!("Error getting connection from redis: {:?}", err);
                None
            }
        }
    }

    pub async fn ping(&self) -> bool {
        match self.get_async_connection().await {
            None => false,
            Some(mut conn) => {
                let result: RedisResult<String> = cmd("PING").query_async(&mut conn).await;
                result.is_ok()
            }
        }
    }
}
