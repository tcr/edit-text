// TODO clean up these imports!

use crate::{
    SyncClientCommand,
    SyncServerCommand,
    db::*,
    util::*,
};

use extern::{
    bus::{Bus, BusReader},
    crossbeam_channel::{
        Receiver as CCReceiver,
        Sender as CCSender,
        unbounded,
    },
    diesel::{
        sqlite::SqliteConnection,
    },
    failure::Error,
    juniper,
    oatie::{
        OT,
        doc::*,
        schema::RtfSchema,
        validate::validate_doc,
    },
    simple_ws::*,
    rand::{thread_rng, Rng},
    r2d2,
    r2d2_diesel::ConnectionManager,
    ron,
    rouille,
    serde_json,
    std::{
        collections::{HashMap, VecDeque},
        sync::{Arc, Mutex},
        thread::{self, JoinHandle},
        time::Duration,
    },
    url::Url,
    ws,
};

use std::io::prelude::*;
use juniper::http::{GraphQLRequest};
use juniper::{FieldResult, EmptyMutation};


#[derive(GraphQLObject)]
struct Page {
    doc: String,
}

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

// Arbitrary context data.
struct Ctx(r2d2::Pool<ConnectionManager<SqliteConnection>>);

// A root schema consists of a query and a mutation.
// Request queries can be executed against a RootNode.
type Schema = juniper::RootNode<'static, Query, EmptyMutation<Ctx>>;

pub fn sync_graphql_server(
    db_pool: r2d2::Pool<ConnectionManager<SqliteConnection>>,
) {
    eprintln!("Graphql served on http://0.0.0.0:8003");
    rouille::start_server("0.0.0.0:8003", move |request| {
        router!(request,
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
                    &Schema::new(Query, EmptyMutation::new()),
                    &ctx,
                );
                rouille::Response::json(&res)
            },

            _ => rouille::Response::empty_404()
        )
    });
}
