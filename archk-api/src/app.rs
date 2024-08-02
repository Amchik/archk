use serde::Deserialize;
use sqlx::SqlitePool;

use crate::roles::UserRoles;

/// Default bcrypt cost for passwords
pub(crate) const BCRYPT_COST: u32 = 13;

#[derive(Deserialize)]
pub struct AppConfig {
    /// Server config
    pub server: AppConfigServer,
}

#[derive(Deserialize)]
pub struct AppConfigServer {
    /// IP and port to server be published
    pub publish_on: String,

    /// Database url
    pub database: String,

    /// User roles
    pub roles: UserRoles,
}

#[derive(Clone)]
pub struct AppState {
    pub db: SqlitePool,
    pub roles: &'static UserRoles,
}
