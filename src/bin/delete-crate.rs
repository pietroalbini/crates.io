// Purge all references to a crate from the database and the index.
//
// Please be super sure you want to do this before running this.
//
// Usage:
//      cargo run --bin delete-crate crate-name

#![warn(clippy::all, rust_2018_idioms)]

use cargo_registry::{db, git, models::Crate};
use std::{
    env,
    io::{self, prelude::*},
};
use swirl::Job;

use diesel::prelude::*;

fn main() {
    let conn = db::connect_now().unwrap();
    conn.transaction::<_, diesel::result::Error, _>(|| {
        delete(&conn);
        Ok(())
    })
    .unwrap()
}

fn delete(conn: &PgConnection) {
    let name = match env::args().nth(1) {
        None => {
            println!("needs a crate-name argument");
            return;
        }
        Some(s) => s,
    };

    let krate = Crate::by_name(&name).first::<Crate>(conn).unwrap();
    print!(
        "Are you sure you want to delete {} ({}) [y/N]: ",
        name, krate.id
    );
    io::stdout().flush().unwrap();
    let mut line = String::new();
    io::stdin().read_line(&mut line).unwrap();
    if !line.starts_with('y') {
        return;
    }

    git::remove_crate(krate.id)
        .enqueue(&conn)
        .expect("failed to enqueue the removal job");
    println!("queued the job to remove the crate from crates.io");
}
