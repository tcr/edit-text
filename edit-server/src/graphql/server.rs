//! GraphQL server.

use crate::{
    db::*,
};

use extern::{
    diesel::{
        sqlite::SqliteConnection,
    },
    juniper::{
        self,
        http::{GraphQLRequest},
        FieldResult,
    },
    oatie::{
        doc::*,
    },
    edit_common::markdown::*,
    r2d2,
    r2d2_diesel::ConnectionManager,
    rouille,
    serde_json,
    std::io::prelude::*,
};

struct Page {
    doc: String,
}

graphql_object!(Page: () |&self| {
    field doc() -> &str {
        self.doc.as_str()
    }

    field markdown() -> String {
        let doc = Doc(::ron::de::from_str(&self.doc).unwrap());
        doc_to_markdown(&doc.0).unwrap()
    }
});

struct Query;

graphql_object!(Query: Ctx |&self| {
    field page(&executor, id: String) -> FieldResult<Option<Page>> {
        let conn = executor.context().0.get().unwrap();

        let page = get_single_page_raw(&conn, &id);

        Ok(page.map(|x| Page {
            doc: x.body
        }))
    }
});

struct Mutations;

graphql_object!(Mutations: Ctx |&self| {
    field createPage(&executor, id: String, doc: String) -> FieldResult<Page> {
        let conn = executor.context().0.get().unwrap();

        let doc = Doc(::ron::de::from_str(&doc).unwrap());
        create_page(&conn, &id, &doc);
        let page = get_single_page_raw(&conn, &id);

        // TODO kick of entries in PageMaster[id] (PageController)
        // with a shutdown query

        Ok(page.map(|x| Page {
            doc: x.body
        }).unwrap())
    }

    field getOrCreatePage(&executor, id: String, default: String) -> FieldResult<Page> {
        let conn = executor.context().0.get().unwrap();

        let doc = get_single_page_raw(&conn, &id)
            .map(|x| x.body)
            .unwrap_or_else(move || {
                let doc = Doc(::ron::de::from_str(&default).unwrap());
                create_page(&conn, &id, &doc);
                default
            });

        Ok(Page {
            doc
        })
    }
});

// Arbitrary context data.
struct Ctx(r2d2::Pool<ConnectionManager<SqliteConnection>>);

// A root schema consists of a query and a mutation.
// Request queries can be executed against a RootNode.
type Schema = juniper::RootNode<'static, Query, Mutations>;

pub fn sync_graphql_server(
    db_pool: r2d2::Pool<ConnectionManager<SqliteConnection>>,
) {
    eprintln!("Graphql served on http://0.0.0.0:8003");
    rouille::start_server("0.0.0.0:8003", move |request| {
        router!(request,
            (OPTIONS) (/graphql/) => {
                rouille::Response::text("")
                    .with_unique_header("Access-Control-Allow-Origin", "*")
                    .with_unique_header("Access-Control-Allow-Headers", "content-type")
            },

            (POST) (/graphql/) => {
                let mut data = request.data().unwrap();
                let mut buf = Vec::new();
                match data.read_to_end(&mut buf) {
                    Ok(_) => {}
                    Err(_) => return rouille::Response::text("Failed to read body"),
                }

                // Create a context object.
                let ctx = Ctx(db_pool.clone());

                // Populate the GraphQL request object.
                let req = match serde_json::from_slice::<GraphQLRequest>(&mut buf) {
                    Ok(value) => value,
                    Err(_) => return rouille::Response::text("Failed to read body"),
                };

                // Run the executor.
                let res = req.execute(
                    &Schema::new(Query, Mutations),
                    &ctx,
                );
                rouille::Response::json(&res)
                    .with_unique_header("Access-Control-Allow-Origin", "*")
                    .with_unique_header("Access-Control-Allow-Headers", "content-type")
            },

            _ => rouille::Response::empty_404()
        )
    });
}
