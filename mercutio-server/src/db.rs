use diesel::{
    self,
    prelude::*,
    sqlite::SqliteConnection,
};
use dotenv::dotenv;
use std::{
    collections::HashMap,
    env,
};

pub fn db_connection() -> SqliteConnection {
    dotenv().ok();

    let mut database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    database_url = format!("../{}", database_url);
    SqliteConnection::establish(&database_url)
        .expect(&format!("Error connecting to {}", database_url))
}

#[derive(Queryable)]
pub struct Post {
    pub id: String,
    pub body: String,
}

use super::schema::posts;

#[derive(Insertable)]
#[table_name = "posts"]
pub struct NewPost<'a> {
    pub id: &'a str,
    pub body: &'a str,
}

pub fn create_post<'a>(conn: &SqliteConnection, id: &'a str, body: &'a str) -> usize {
    use super::schema::posts;

    let new_post = NewPost { id: id, body: body };

    diesel::replace_into(posts::table)
        .values(&new_post)
        .execute(conn)
        .expect("Error saving new post")
}

pub fn db_help() -> (SqliteConnection, HashMap<String, String>) {
    use super::schema::posts::dsl::*;

    let connection = db_connection();
    let results = posts
        // .limit(5)
        .load::<Post>(&connection)
        .expect("Error loading posts");

    // create_post(&connection, "home", "# hello world");

    // println!("Displaying {} posts", results.len());
    let mut ret = HashMap::new();
    for post in results {
        ret.insert(post.id.clone(), post.body.clone());
        // println!("{}", post.id);
        // println!("----------\n");
        // println!("{}", post.body);
    }
    (connection, ret)
}
