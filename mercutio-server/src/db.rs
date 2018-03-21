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

#[derive(Queryable, Debug)]
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

pub fn all_posts(db: &SqliteConnection) -> HashMap<String, String> {
    use super::schema::posts::dsl::*;

    let results = posts
        .load::<Post>(db)
        .expect("Error loading posts");

    let mut ret = HashMap::new();
    for post in results {
        ret.insert(post.id.clone(), post.body.clone());
    }
    ret
}

pub fn get_single_page(db: &SqliteConnection, input_id: &str) -> Option<Post> {
    use super::schema::posts::dsl::*;

    return posts
        .filter(id.eq(input_id))
        .first::<Post>(db)
        .ok();
}

