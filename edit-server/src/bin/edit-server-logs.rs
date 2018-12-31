use diesel::connection::Connection;
use edit_server::db::*;
use failure::*;
use serde_json;
use structopt::*;

#[derive(Debug, StructOpt)]
enum Cli {
    #[structopt(name = "list")]
    List {
        #[structopt(long = "source")]
        source: Option<String>,
    },

    #[structopt(name = "clear")]
    Clear,
}

fn main() -> Result<(), Error> {
    let args = Cli::from_args();

    let db = db_connection();

    match args {
        Cli::Clear => {
            clear_all_logs(&db)?;
            db.execute("VACUUM").unwrap();
            eprintln!("cleared logs.");
        }
        Cli::List { source } => {
            let logs = if let Some(source) = source {
                eprintln!("Filter by source: {}", source);
                select_logs(&db, &source)?
            } else {
                all_logs(&db)?
            };

            eprintln!("Printing {} logs...", logs.len());

            for log in logs {
                println!("{}", serde_json::to_string(&log).unwrap());
            }
        }
    }

    Ok(())
}
