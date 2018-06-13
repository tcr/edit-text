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

#[derive(Queryable, Debug)]
pub struct Log {
    pub source: String,
    pub body: String,
}

use super::schema::logs;

#[derive(Insertable)]
#[table_name = "logs"]
pub struct NewLog<'a> {
    pub source: &'a str,
    pub body: &'a str,
}
