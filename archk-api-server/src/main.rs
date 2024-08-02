use std::{fs, net::SocketAddrV4};

use archk_api::app::{AppConfig, AppConfigServerPublishOnPort, AppState};
use axum::{routing::get, Router};
use sqlx::SqlitePool;

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

    if let Err(err) = archk_api::apply_migrations(&db).await {
        eprintln!("Failed to migrate on `{}`: {err}", config.database);
        panic!("failed to migrate: {err}");
    }

    let state = AppState {
        db,
        roles: Box::leak(Box::new(config.roles)),
    };

    let app = Router::new()
        .nest("/api/v1", archk_api::v1::get_routes())
        .route("/", get(|| async { String::from("hi") }))
        .with_state(state);

    let port = match config.publish_on.port {
        AppConfigServerPublishOnPort::Port(v) => v,
        AppConfigServerPublishOnPort::ObtainFromEnv => {
            match std::env::var("PORT").map(|v| v.parse()) {
                Ok(Ok(v)) => v,
                _ => {
                    eprintln!("You should pass valid port in `$PORT` envinroment variable or change `server.publish_on` option in config");
                    panic!("failed to obtain port from env ($PORT)");
                }
            }
        }
    };

    let listener =
        tokio::net::TcpListener::bind(SocketAddrV4::new(config.publish_on.ip, port)).await;
    let listener = match listener {
        Ok(v) => v,
        Err(err) => {
            eprintln!(
                "Failed to bind to address `{}:{}`: {err}",
                config.publish_on.ip, port
            );
            panic!("failed to bind: {err}");
        }
    };

    tracing::info!(
        ip = config.publish_on.ip.to_string(),
        port = port,
        "Starting server"
    );
    axum::serve(listener, app).await.unwrap();
}
