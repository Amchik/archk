use archk::v1::{
    api,
    auth::{Token, TokenTy},
    service::{ServiceAccountID, ServiceAccountTy},
    space::SpaceID,
    user::UserID,
};
use axum::{
    async_trait,
    extract::FromRequestParts,
    http::{header::AUTHORIZATION, request::Parts, HeaderMap},
};

use crate::app::AppState;

#[derive(Debug)]
pub struct DbUser {
    pub id: String,
    pub name: String,
    pub invites: i64,
    pub invited_by: Option<String>,
    pub level: i64,
    pub password_hash: String,
}

#[derive(Debug)]
#[allow(dead_code)] // ???
pub struct DbService {
    pub id: ServiceAccountID,
    pub space_id: Option<SpaceID>,
    pub ty: ServiceAccountTy,
}

#[async_trait]
pub trait AuthenticatedUserParam: Sized {
    async fn verify(token: &Token, state: &AppState) -> Option<Self>;
}

pub struct AuthenticatedUser<U: AuthenticatedUserParam = UserID> {
    pub token: Token,
    pub user: U,
}

#[async_trait]
impl AuthenticatedUserParam for UserID {
    async fn verify(token: &Token, state: &AppState) -> Option<Self> {
        if token.ty != TokenTy::Personal {
            return None;
        }

        let iat = token.iat as i64;
        let rnd = token.rnd as i64;
        let res = sqlx::query!(
            "SELECT user_id FROM tokens WHERE iat = ? AND rnd = ?",
            iat,
            rnd
        )
        .fetch_optional(&state.db)
        .await
        .expect("database");

        res.map(|v| {
            UserID::from(v.user_id)
                .expect("Invalid user id from database in AuthenticatedUser::from_request_parts")
        })
    }
}

#[async_trait]
impl AuthenticatedUserParam for DbUser {
    async fn verify(token: &Token, state: &AppState) -> Option<Self> {
        if token.ty != TokenTy::Personal {
            return None;
        }

        let iat = token.iat as i64;
        let rnd = token.rnd as i64;

        sqlx::query_as!(
            DbUser,
            "SELECT users.* FROM users INNER JOIN tokens ON tokens.user_id = users.id WHERE tokens.iat = ? AND tokens.rnd = ?",
            iat,
            rnd
        )
        .fetch_optional(&state.db)
        .await
        .expect("database")
    }
}

#[async_trait]
impl AuthenticatedUserParam for DbService {
    async fn verify(token: &Token, state: &AppState) -> Option<Self> {
        if token.ty != TokenTy::Service {
            return None;
        }

        let iat = token.iat as i64;
        let rnd = token.rnd as i64;

        let res = sqlx::query!(
            "
                SELECT
                    service_tokens.service_id as id,
                    service_accounts.space_id,
                    service_accounts.ty
                FROM service_tokens
                    INNER JOIN service_accounts
                        ON service_tokens.service_id = service_accounts.id
                WHERE service_tokens.iat = ? AND service_tokens.rnd = ?",
            iat,
            rnd
        )
        .fetch_optional(&state.db)
        .await
        .expect("database")?;

        Some(Self {
            id: ServiceAccountID::from(res.id)?,
            space_id: res.space_id.map(SpaceID::from).flatten(),
            ty: ServiceAccountTy::try_from(res.ty).ok()?,
        })
    }
}

#[async_trait]
impl<U: AuthenticatedUserParam> FromRequestParts<AppState> for AuthenticatedUser<U> {
    type Rejection = api::Response;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let headers = HeaderMap::from_request_parts(parts, state)
            .await
            .map_err(|err| match err {})?;

        let token_str = headers
            .get(AUTHORIZATION)
            .and_then(|v| v.to_str().ok())
            .filter(|v| v.starts_with("Bearer "))
            .map(|v| &v[("Bearer ".len())..]);

        let Some(Ok(token)) = token_str.map(Token::parse) else {
            return Err(api::Response::Failture(api::Error::Unauthorized.detail(
                "Expected valid user token in header `Authorization: Bearer <TOKEN>`".into(),
            )));
        };

        let user = <U as AuthenticatedUserParam>::verify(&token, state).await;

        match user {
            Some(user) => Ok(Self { token, user }),
            None => Err(api::Response::Failture(
                api::Error::Unauthorized.detail("Unknown token".into()),
            )),
        }
    }
}
