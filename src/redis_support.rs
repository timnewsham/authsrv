
use std::time::Duration;
use redis;
use r2d2;
use r2d2_redis;
use rocket::{Build, Rocket};
use rocket_sync_db_pools::{Poolable, Error, Config};

#[allow(type_alias_bounds)]
pub type PoolResult<P: Poolable> = Result<r2d2::Pool<P::Manager>, Error<P::Error>>;

impl Poolable for redis::Connection {
    type Manager = r2d2_redis::RedisConnectionManager;
    type Error = redis::RedisError;

    fn pool(db_name: &str, rocket: &Rocket<Build>) -> PoolResult<Self> {
        let config = Config::from(db_name, rocket)?;
        let manager = r2d2_redis::RedisConnectionManager::new(&*config.url);
        let pool = r2d2::Pool::builder()
            .max_size(config.pool_size)
            .connection_timeout(Duration::from_secs(config.timeout as u64))
            .build(manager)?;

        Ok(pool)
    }
}
