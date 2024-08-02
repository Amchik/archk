use std::fs;

use sqlx::migrate::Migrator;
use sqlx::SqlitePool;

#[tokio::main]
async fn main() {
    println!("cargo::rerun-if-changed=migrations/");

    // relative path
    _ = fs::File::options()
        .read(true)
        .write(true)
        .create(true)
        .open("../archk.db")
        .map(drop);

    let db = SqlitePool::connect("sqlite://../archk.db")
        .await
        .expect("db connection");

    let m = Migrator::new(std::path::Path::new("./migrations"))
        .await
        .expect("migrator");

    m.run(&db).await.expect("migration");
}
