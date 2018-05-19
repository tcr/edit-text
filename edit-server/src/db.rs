use diesel::{prelude::*, sqlite::SqliteConnection};
use dotenv::dotenv;
use oatie::doc::*;
use r2d2;
use r2d2_diesel::ConnectionManager;
use std::env;

pub mod queries;
pub mod schema;

pub use self::queries::*;

pub fn db_pool_create() -> r2d2::Pool<ConnectionManager<SqliteConnection>> {
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

pub type DbPool = r2d2::Pool<ConnectionManager<SqliteConnection>>;

#[derive(Queryable, Debug)]
pub struct Post {
    pub id: String,
    pub body: String,
}

use self::schema::posts;

#[derive(Insertable)]
#[table_name = "posts"]
pub struct NewPost<'a> {
    pub id: &'a str,
    pub body: &'a str,
}
