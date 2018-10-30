use crate::db::*;

use diesel::{
    self,
    sqlite::SqliteConnection,
};
use failure::Error;
use std::collections::HashMap;

fn lock_retry<T, F>(mut f: F) -> Result<T, diesel::result::Error>
where
    F: FnMut() -> Result<T, diesel::result::Error>,
{
    loop {
        let res = f();
        if let Err(ref err) = res {
            let err = format!("{:?}", err);
            if err.find("database is locked").is_some() {
                ::std::thread::sleep(::std::time::Duration::from_millis(10));
                continue;
            }
        }
        return res;
    }
}

// TODO usize is not useful.
// also is this always upsert? shoudl be named that then
pub fn create_page<'a>(conn: &SqliteConnection, id: &'a str, doc: &Doc) -> usize {
    use super::schema::posts;

    let body = ::ron::ser::to_string(&doc.0).unwrap();

    let new_post = NewPost {
        id: id,
        body: &body,
    };

    lock_retry(|| {
        diesel::replace_into(posts::table)
            .values(&new_post)
            .execute(conn)
    })
    .expect("Error saving new post")
}

pub fn all_posts(db: &SqliteConnection) -> HashMap<String, String> {
    use super::schema::posts::dsl::*;

    let results = lock_retry(|| posts.load::<Post>(db)).expect("Error loading posts");

    let mut ret = HashMap::new();
    for post in results {
        ret.insert(post.id.clone(), post.body.clone());
    }
    ret
}

pub fn get_single_page(db: &SqliteConnection, input_id: &str) -> Option<Doc> {
    use super::schema::posts::dsl::*;

    let post = lock_retry(|| posts.filter(id.eq(input_id)).first::<Post>(db));

    post.map_err::<Error, _>(|x| x.into())
        // HACK strip null bytes that have snuck into the database
        .map(|x| {
            if x.body.find(r"\u{0}").is_some() {
                eprintln!("(!) Stripped NUL byte from doc.");
                x.body.replace(r"\u{0}", "")
            } else {
                x.body.to_string()
            }
        })
        .and_then(|x| Ok(::ron::de::from_str::<DocSpan>(&x)?))
        .map(|d| Doc(d))
        .ok()
}

pub fn get_single_page_raw(db: &SqliteConnection, input_id: &str) -> Option<Post> {
    use super::schema::posts::dsl::*;

    lock_retry(|| posts.filter(id.eq(input_id)).first::<Post>(db)).ok()
}

// Logs

pub fn create_log<'a>(
    conn: &SqliteConnection,
    source: &'a str,
    body: &'a str,
) -> Result<usize, Error> {
    use super::schema::logs;

    let new_log = NewLog {
        source: source,
        body: &body,
    };

    Ok(lock_retry(|| {
        diesel::insert_into(logs::table)
            .values(&new_log)
            .execute(conn)
    })?)
}

pub fn all_logs(db: &SqliteConnection) -> Result<Vec<Log>, Error> {
    use super::schema::logs::dsl::*;

    Ok(lock_retry(|| logs.load::<Log>(db))?)
}

pub fn select_logs(db: &SqliteConnection, input_source: &str) -> Result<Vec<Log>, Error> {
    use super::schema::logs::dsl::*;

    Ok(lock_retry(|| {
        logs.filter(source.eq(input_source)).load(db)
    })?)
}

pub fn clear_all_logs(db: &SqliteConnection) -> Result<usize, Error> {
    use super::schema::logs::dsl::*;

    Ok(lock_retry(|| diesel::delete(logs).execute(db))?)
}
