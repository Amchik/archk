use std::fs;

use app::{AppConfig, AppState};
use axum::{routing::get, Router};
use sqlx::SqlitePool;

mod app;
mod roles;
mod v1;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let config = {
        let cfg_path = std::env::var("CONFIG_PATH").unwrap_or("config.yml".into());
        let cfg = match fs::read_to_string(&cfg_path) {
            Ok(cfg) => cfg,
            Err(e) => {
                eprintln!(
                    "Failed to read config `{cfg_path}` (got from `$CONFIG_PATH` variable): {e}"
                );
                eprintln!("help: config example located in source repository `config.example.yml`");
                panic!("failed to read config: {e}");
            }
        };
        let cfg: AppConfig = match serde_yaml::from_str(&cfg) {
            Ok(v) => v,
            Err(e) => {
                eprintln!("Failed to deserialize config `{cfg_path}` (got from `$CONFIG_PATH$` variable): {e}");
                eprintln!("help: config example located in source repository `config.example.yml`");
                if let Some(loc) = e.location() {
                    let (line, col) = (loc.line(), loc.column());
                    eprintln!("help: failed on line {line} column {col}");
                    let line_no_str = line.to_string();
                    let line_str = cfg.lines().skip(line - 1).next().unwrap_or_default();
                    eprintln!(" {line_no_str} | {line_str}");
                    (0..line_no_str.len() + 3 + col).for_each(|_| eprint!(" "));
                    eprintln!("^ {e}");
                }
                panic!("failed to deserialize config: {e}");
            }
        };
        cfg.server
    };

    let db = SqlitePool::connect(&config.database)
        .await
        .expect("db connection");

    if let Err(err) = sqlx::migrate!().run(&db).await {
        eprintln!("Failed to migrate on `{}`: {err}", config.database);
        panic!("failed to migrate: {err}");
    }

    let state = AppState {
        db,
        roles: Box::leak(Box::new(config.roles)),
    };

    let app = Router::new()
        .nest("/api/v1", v1::get_routes())
        .route("/", get(|| async { String::from("hi") }))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind(&config.publish_on).await;
    let listener = match listener {
        Ok(v) => v,
        Err(err) => {
            eprintln!("Failed to bind to address `{}`: {err}", config.publish_on);
            panic!("failed to bind: {err}");
        }
    };

    tracing::info!(publish_on = config.publish_on, "Server staring");
    axum::serve(listener, app).await.unwrap();
}
