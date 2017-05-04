use diesel::pg::PgConnection;
use r2d2::{Pool, Config};
use r2d2_diesel::ConnectionManager;

use ENV;

pub fn create_db_pool() -> Pool<ConnectionManager<PgConnection>> {
    let database_url = ENV.database_url.as_ref();
    let config = Config::default();
    let manager = ConnectionManager::<PgConnection>::new(database_url);
    Pool::new(config, manager).expect("Failed to create pool.")
}

macro_rules! get_conn {
    ($pool: expr) => (
       $pool.0.get().or_else(|_| Err(error::internal_server_error("database connection pool timeout")))
    )
}
