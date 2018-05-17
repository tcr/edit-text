use crate::{
    db::*,
};

use extern::{
    diesel::{
        self,
        sqlite::SqliteConnection,
    },
    std::{
        collections::HashMap,
    },
    failure::Error,
};

// TODO usize is not useful.
// also is this always upsert? shoudl be named that then
pub fn create_page<'a>(conn: &SqliteConnection, id: &'a str, doc: &Doc) -> usize {
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

pub fn get_single_page_raw(db: &SqliteConnection, input_id: &str) -> Option<Post> {
    use super::schema::posts::dsl::*;

    return posts
        .filter(id.eq(input_id))
        .first::<Post>(db)
        .ok()
}

