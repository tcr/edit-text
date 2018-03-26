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
use failure::Error;
use oatie::doc::*;

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

pub fn create_post<'a>(conn: &SqliteConnection, id: &'a str, doc: &Doc) -> usize {
    use super::schema::posts;

    let body = ::ron::ser::to_string(&doc.0).unwrap();

    let new_post = NewPost { id: id, body: &body };

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

pub fn get_single_page(db: &SqliteConnection, input_id: &str) -> Option<Doc> {
    use super::schema::posts::dsl::*;

    return posts
        .filter(id.eq(input_id))
        .first::<Post>(db)
        .map_err::<Error, _>(|x| x.into())
        .and_then(|x| Ok(::ron::de::from_str::<DocSpan>(&x.body)?))
        .map(|d| Doc(d))
        .ok()
}

