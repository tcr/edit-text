//! GraphQL server.

use crate::{
    db::*,
    sync::{
        ClientUpdate,
        ClientNotify,
    },
};

use extern::{
    crossbeam_channel::{
        Sender as CCSender,
    },
    diesel::{
        sqlite::SqliteConnection,
    },
    juniper::{
        self,
        http::{GraphQLRequest},
        FieldResult,
        FieldError,
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
        let conn = executor.context().db_pool.get().unwrap();

        let page = get_single_page_raw(&conn, &id);

        Ok(page.map(|x| Page {
            doc: x.body
        }))
    }
});

struct Mutations;

graphql_object!(Mutations: Ctx |&self| {
    // TODO rename this to upsert
    field createPage(
        &executor, 
        id: String,
        doc: Option<String>,
        markdown: Option<String>,
    ) -> FieldResult<Page> {
        let doc = match (markdown, doc) {
            (None, None) => {
                return Err(FieldError::new(
                    "Must specify one of doc or markdown",
                    juniper::Value::null(),
                ));
            }
            (_, Some(doc)) => {
                Doc(::ron::de::from_str(&doc).unwrap())
            }
            (Some(markdown), _) => {
                Doc(markdown_to_doc(&markdown).unwrap())
            }
        };

        // Get db connection.
        let conn = executor.context().db_pool.get().unwrap();

        // Create the page, store in database, and restore.
        create_page(&conn, &id, &doc);
        let page = get_single_page_raw(&conn, &id);

        // Kick off all current clients.
        let _ = executor.context().tx_master.send(ClientNotify(id.clone(), ClientUpdate::Overwrite {
            doc,
        }));

        // TODO there is probably a race condition between create_page and the overwrite
        // kicking off users from sync. This should be fixed

        // TODO can the below executor code in getOrCreatePage also be the same code here?

        Ok(page.map(|x| Page {
            doc: x.body
        }).unwrap())
    }

    field getOrCreatePage(
        &executor, 
        id: String, 
        default: String,
    ) -> FieldResult<Page> {
        let conn = executor.context().db_pool.get().unwrap();

        let doc = get_single_page_raw(&conn, &id)
            .map(|x| x.body)
            .unwrap_or_else(move || {
                let doc = Doc(::ron::de::from_str(&default).unwrap());
                create_page(&conn, &id, &doc);

                let _ = executor.context().tx_master.send(ClientNotify(id.clone(), ClientUpdate::Overwrite {
                    doc,
                }));
                
                default
            });

        Ok(Page {
            doc
        })
    }
});

// Arbitrary context data.
#[derive(Clone)]
struct Ctx {
    db_pool: r2d2::Pool<ConnectionManager<SqliteConnection>>,
    tx_master: CCSender<ClientNotify>,
}

// A root schema consists of a query and a mutation.
// Request queries can be executed against a RootNode.
type Schema = juniper::RootNode<'static, Query, Mutations>;

pub fn sync_graphql_server(
    db_pool: r2d2::Pool<ConnectionManager<SqliteConnection>>,
    tx_master: CCSender<ClientNotify>,
) {
    // Create a context object.
    let ctx = Ctx {
        db_pool,
        tx_master,
    };
    
    eprintln!("Graphql served on http://0.0.0.0:8003");
    rouille::start_server("0.0.0.0:8003", move |request| {
        let ctx = ctx.clone();

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
