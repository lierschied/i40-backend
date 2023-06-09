//! # migrate
//!
//! `migrate` is a helper binary to execute migration scripts for the database
//!
//! # Example
//! Running a migration for a .surql file.
//!
//! ```text
//! # Single file
//! > cargo run -p migrate -- my_first_migration
//!
//! # Multiple files
//! > cargo run -p migrate -- my_second_migration my_third_migration
//! ```
//!

use std::{
    env::args,
    fs::File,
    io::{BufReader, Read},
};

use surrealdb::{engine::remote::ws::Ws, opt::auth::Root, Surreal};

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let db = Surreal::new::<Ws>("127.0.0.1:8000")
        .await
        .expect("Unable to connect to database");

    db.signin(Root {
        username: "root",
        password: "root",
    })
    .await
    .unwrap();

    for filename in args().skip(1) {
        let filename = format!("./web/surql/{filename}.surql");
        let file = File::open(&filename)?;
        let mut content = String::new();
        let _ = BufReader::new(file).read_to_string(&mut content);

        println!("Executing: {filename}");
        db.query(content).await.unwrap();
    }

    if args().skip(1).len() > 0 {
        println!("Executed Query count: {}", args().skip(1).len());
        println!("Done!");
    } else {
        println!("Please specify a migration to run!");
    }

    Ok(())
}
