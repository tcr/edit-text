use diesel::{prelude::*, sqlite::SqliteConnection};
use dotenv::dotenv;
use oatie::doc::*;
use r2d2;
use r2d2_diesel::ConnectionManager;
use std::env;

pub mod queries;
pub mod schema;
pub mod types;

pub use self::queries::*;
pub use self::types::*;

pub type DbPool = r2d2::Pool<ConnectionManager<SqliteConnection>>;

pub fn db_pool_create() -> DbPool {
    dotenv().ok();

    let mut database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    database_url = format!("../{}", database_url);

    let manager = ConnectionManager::<SqliteConnection>::new(database_url.clone());
    r2d2::Pool::builder()
        .build(manager)
        .expect(&format!("Error connecting to {}", database_url))
}

pub fn db_connection() -> SqliteConnection {
    dotenv().ok();

    let mut database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    database_url = format!("../{}", database_url);

    SqliteConnection::establish(&database_url)
        .expect(&format!("Error connecting to {}", database_url))
}
