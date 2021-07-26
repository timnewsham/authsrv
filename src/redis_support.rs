
/*
 * Borrows heavily from https://github.com/sorccu/r2d2-redis/blob/master/src/lib.rs
 * but I cant use r2d2 directly because of reasons...
 */
use std::time::Duration;
use redis;
use redis::ConnectionLike;
use r2d2;
use rocket::{Build, Rocket};
use rocket_sync_db_pools::{Poolable, Error, Config};

#[allow(type_alias_bounds)]
pub type PoolResult<P: Poolable> = Result<r2d2::Pool<P::Manager>, Error<P::Error>>;

pub struct Connection(pub redis::Connection);

pub struct ConnectionManager {
    connection_info: redis::ConnectionInfo,
}

impl ConnectionManager {
    pub fn new<T: redis::IntoConnectionInfo>(params: T) -> Result<ConnectionManager, redis::RedisError> {
        Ok(ConnectionManager {
            connection_info: params.into_connection_info()?,
        })
    }
}

impl r2d2::ManageConnection for ConnectionManager {
    type Connection = Connection;
    type Error = redis::RedisError;
    fn connect(&self) -> Result<Self::Connection, Self::Error> {
        match redis::Client::open(self.connection_info.clone()) {
            Ok(client) => client.get_connection().map(|c| Connection(c)),
            Err(err) => Err(err)
        }
    }
    fn is_valid(&self, conn: &mut Self::Connection) -> Result<(), Self::Error> {
        redis::cmd("PING").query(&mut conn.0)
    }
    fn has_broken(&self, conn: &mut Self::Connection) -> bool {
        !conn.0.is_open()
    }
}

impl Poolable for Connection {
    type Manager = ConnectionManager;
    type Error = redis::RedisError;

    fn pool(db_name: &str, rocket: &Rocket<Build>) -> PoolResult<Self> {
        let config = Config::from(db_name, rocket)?;
        let manager = ConnectionManager::new(&*config.url).map_err(Error::Custom)?;
        let pool = r2d2::Pool::builder()
            .max_size(config.pool_size)
            .connection_timeout(Duration::from_secs(config.timeout as u64))
            .build(manager)?;

        Ok(pool)
    }
}
