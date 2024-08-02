use archk::{
    v1::{
        api::{self, Response},
        auth::{Token, TokenTy},
        user::is_valid_username,
    },
    Documentation,
};
use axum::{extract::State, Json};
use serde::{Deserialize, Serialize};

use crate::app::AppState;

#[derive(Deserialize, Documentation)]
pub struct AuthorizationRequestData {
    /// User name
    pub username: String,
    /// User password
    pub password: String,
}

#[derive(Serialize, Documentation)]
pub struct AuthorizationResponse {
    /// Bearer token
    pub token: String,
}

pub async fn authorize(
    State(AppState { db, .. }): State<AppState>,
    Json(AuthorizationRequestData { username, password }): Json<AuthorizationRequestData>,
) -> Response<AuthorizationResponse> {
    if !is_valid_username(&username) {
        return Response::Failture(api::Error::MalformedData.detail("Invalid username".into()));
    }

    let (id, password_hash) = {
        let stmt = sqlx::query!(
            "SELECT id, password_hash FROM users WHERE name = ?",
            username
        )
        .fetch_one(&db)
        .await;
        match stmt {
            Ok(v) => (v.id, v.password_hash),
            Err(_) => return Response::Failture(api::Error::ObjectNotFound.into()),
        }
    };

    if !bcrypt::verify(&password, &password_hash).unwrap_or(false) {
        return Response::Failture(api::Error::ObjectNotFound.into());
    }

    let token = Token::new(TokenTy::Personal);
    let iat = token.iat as i64;
    let rnd = token.rnd as i64;
    let stmt = sqlx::query!(
        "INSERT INTO tokens(iat, rnd, user_id) VALUES (?, ?, ?)",
        iat,
        rnd,
        id
    )
    .execute(&db)
    .await;

    match stmt {
        Ok(_) => Response::Success(AuthorizationResponse {
            token: token.to_string(),
        }),
        Err(_) => Response::Failture(api::Error::Internal.into()),
    }
}
