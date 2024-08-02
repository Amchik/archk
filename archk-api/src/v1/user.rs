use archk::v1::{
    api::{self, Response},
    auth::{Token, TokenTy},
    user::{is_valid_username, User, UserID},
};
use axum::{
    extract::{Path, Query, State},
    Json,
};
use rand::{distributions::Alphanumeric, Rng};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{app::AppState, roles::UserRole};

use super::extra::{AuthenticatedUser, DbUser};

#[derive(Deserialize)]
pub struct RegisterRequestData {
    pub username: String,
    pub password: String,
    pub invite: String,
}

#[derive(Deserialize)]
pub struct InviteWaveData {
    #[serde(default)]
    pub min_level: i64,
}

#[derive(Deserialize)]
pub struct PatchUser {
    pub old_password: String,
    pub new_password: String,
    #[serde(default)]
    pub logout: bool,
}

#[derive(Deserialize)]
pub struct UserIDPath {
    pub user_id: String,
}

#[derive(Deserialize)]
pub struct Paging {
    #[serde(default)]
    pub page: u32,
}

#[derive(Deserialize)]
pub struct PromoteUserBody {
    pub level: i64,
}

#[derive(Serialize)]
pub struct RegisterResponse {
    pub user: User,
    pub token: String,
}

#[derive(Serialize)]
pub struct SelfResponse {
    pub user: User,
    pub invites: i64,
    pub level: i64,
}

#[derive(Serialize)]
pub struct ResetPasswordResponse {
    pub password: String,
    pub tokens_reset: u64,
}

#[derive(Serialize)]
pub struct UserSpaceResponse {
    pub id: String,
    pub title: String,
}

pub async fn get_users(
    _: AuthenticatedUser,
    Query(Paging { page }): Query<Paging>,
    State(AppState { db, .. }): State<AppState>,
) -> Response<Vec<User>> {
    let (offset, limit) = ((page as i64) * 50, 50);

    let res = sqlx::query!(
        "SELECT id, name, invited_by FROM users LIMIT ? OFFSET ?",
        limit,
        offset
    )
    .fetch_all(&db)
    .await
    .expect("database");

    let res = res
        .into_iter()
        .map(|v| User {
            id: UserID::from(v.id).expect("checked UserID"),
            name: v.name,
            invited_by: v.invited_by,
        })
        .collect();

    Response::Success(res)
}

pub async fn get_self(
    AuthenticatedUser {
        user:
            DbUser {
                id,
                name,
                invites,
                invited_by,
                level,
                ..
            },
        ..
    }: AuthenticatedUser<DbUser>,
) -> Response<SelfResponse> {
    Response::Success(SelfResponse {
        user: User {
            id: UserID::from(id).expect("checked UserID unwrap"),
            name,
            invited_by,
        },
        invites,
        level,
    })
}

pub async fn get_user(
    _: AuthenticatedUser<UserID>,
    Path(UserIDPath { user_id }): Path<UserIDPath>,
    State(AppState { db, .. }): State<AppState>,
) -> Response<User> {
    let user = sqlx::query!("SELECT name, invited_by FROM users WHERE id = ?", user_id)
        .fetch_optional(&db)
        .await
        .expect("database");

    match user {
        Some(v) => Response::Success(User {
            id: UserID::from(user_id).expect("checked(db) UserID::from"),
            name: v.name,
            invited_by: v.invited_by,
        }),
        None => Response::Failture(api::Error::ObjectNotFound.into()),
    }
}

pub async fn register(
    State(AppState { db, roles, .. }): State<AppState>,
    Json(RegisterRequestData {
        username,
        password,
        invite,
    }): Json<RegisterRequestData>,
) -> Response<RegisterResponse> {
    // 1. verify input data (but unique keys)
    // 2. try to get invite
    // 3. try to create user (and check for unique keys)
    // 4. create token
    // 5. drop invite
    if !is_valid_username(&username) || !matches!(password.len(), 8..=32) {
        return Response::Failture(
            api::Error::MalformedData.detail("Invalid username or password".into()),
        );
    }

    let invited_by = if invite.is_empty() {
        sqlx::query!("SELECT COUNT(1) as cnt FROM users LIMIT 1")
            .fetch_one(&db)
            .await
            .into_iter()
            .filter(|v| v.cnt == 0)
            .map(|_| None)
            .next()
    } else {
        sqlx::query!("SELECT owner_id FROM invites WHERE id = ?", invite)
            .fetch_optional(&db)
            .await
            .expect("database")
            .map(|v| v.owner_id)
    };

    let Some(invited_by) = invited_by else {
        return Response::Failture(api::Error::ObjectNotFound.detail("Invalid invite".into()));
    };

    let password = bcrypt::hash(password, crate::app::BCRYPT_COST).expect("bcrypt");
    let user_id = UserID::new();
    let user_id_str: &str = &user_id;

    let level = invite
        .is_empty()
        .then_some(0)
        .map(|_| roles.get_max().level)
        .unwrap_or(0);

    let res = sqlx::query!(
        "INSERT INTO users(id, name, invited_by, level, password_hash) VALUES (?, ?, ?, ?, ?)",
        user_id_str,
        username,
        invited_by,
        level,
        password
    )
    .execute(&db)
    .await;

    match res {
        Err(sqlx::Error::Database(v)) if v.is_unique_violation() => {
            return Response::Failture(
                api::Error::Conflict.detail("`username` should be unique".into()),
            )
        }
        _ => res.expect("database"),
    };

    let token = Token::new(TokenTy::Personal);
    let token_str = token.to_string();

    let iat = token.iat as i64;
    let rnd = token.rnd as i64;
    sqlx::query!(
        "INSERT INTO tokens(iat, rnd, user_id) VALUES (?, ?, ?)",
        iat,
        rnd,
        user_id_str
    )
    .execute(&db)
    .await
    .expect("database");

    if !invite.is_empty() {
        sqlx::query!("DELETE FROM invites WHERE id = ?", invite)
            .execute(&db)
            .await
            .expect("database");
    }

    Response::Success(RegisterResponse {
        user: User {
            id: user_id,
            name: username,
            invited_by,
        },
        token: token_str,
    })
}

pub async fn patch_user(
    AuthenticatedUser {
        user: DbUser {
            id: user_id,
            password_hash,
            ..
        },
        token,
    }: AuthenticatedUser<DbUser>,
    State(AppState { db, .. }): State<AppState>,
    Json(PatchUser {
        old_password,
        new_password,
        logout,
    }): Json<PatchUser>,
) -> Response<u64> {
    if !matches!(new_password.len(), 3..=32) {
        return Response::Failture(api::Error::MalformedData.detail("Invalid new password".into()));
    }

    if !bcrypt::verify(old_password, &password_hash).unwrap_or(false) {
        return Response::Failture(api::Error::MalformedData.detail("Invalid password".into()));
    }

    let new_password = bcrypt::hash(new_password, crate::app::BCRYPT_COST).expect("bcrypt");
    sqlx::query!(
        "UPDATE users SET password_hash = ? WHERE id = ?",
        new_password,
        user_id
    )
    .execute(&db)
    .await
    .expect("database");

    if logout {
        let iat = token.iat as i64;
        let rnd = token.rnd as i64;
        let res = sqlx::query!(
            "DELETE FROM tokens WHERE user_id = ? AND iat != ? AND rnd != ?",
            user_id,
            iat,
            rnd
        )
        .execute(&db)
        .await
        .expect("database");

        Response::Success(res.rows_affected())
    } else {
        Response::Success(0)
    }
}

pub async fn reset_user_password(
    Path(UserIDPath { user_id }): Path<UserIDPath>,
    AuthenticatedUser {
        user: DbUser { level, .. },
        ..
    }: AuthenticatedUser<DbUser>,
    State(AppState { db, roles, .. }): State<AppState>,
) -> Response<ResetPasswordResponse> {
    if roles
        .get_current(level)
        .filter(|v| v.permissions.manage)
        .is_none()
    {
        return Response::Failture(api::Error::Forbidden.into());
    }

    let password: String = rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(12)
        .map(char::from)
        .collect();

    let password_hash = bcrypt::hash(&password, crate::app::BCRYPT_COST).expect("bcrypt");
    let res = sqlx::query!(
        "UPDATE users SET password_hash = ? WHERE id = ?",
        password_hash,
        user_id
    )
    .execute(&db)
    .await
    .expect("database");

    if res.rows_affected() == 0 {
        return Response::Failture(api::Error::ObjectNotFound.into());
    }

    let res = sqlx::query!("DELETE FROM tokens WHERE user_id = ?", user_id)
        .execute(&db)
        .await
        .expect("database");

    Response::Success(ResetPasswordResponse {
        password,
        tokens_reset: res.rows_affected(),
    })
}

pub async fn get_invites(
    AuthenticatedUser { user, .. }: AuthenticatedUser<UserID>,
    State(AppState { db, .. }): State<AppState>,
) -> Response<Vec<String>> {
    let user_str: &str = &user;
    let invites = sqlx::query!(
        "SELECT id FROM invites WHERE owner_id = ? LIMIT 50",
        user_str
    )
    .fetch_all(&db)
    .await
    .expect("database")
    .into_iter()
    .map(|v| v.id)
    .collect();

    Response::Success(invites)
}

pub async fn create_invite(
    AuthenticatedUser {
        user: DbUser {
            id: user_id,
            invites,
            ..
        },
        ..
    }: AuthenticatedUser<DbUser>,
    State(AppState { db, .. }): State<AppState>,
) -> Response<String> {
    if invites <= 0 {
        return Response::Failture(api::Error::Forbidden.into());
    }

    let invite_id = Uuid::new_v4().to_string();
    sqlx::query!(
        "INSERT INTO invites(id, owner_id) VALUES (?, ?)",
        invite_id,
        user_id
    )
    .execute(&db)
    .await
    .expect("database");

    sqlx::query!(
        "UPDATE users SET invites = invites - 1 WHERE id = ?",
        user_id
    )
    .execute(&db)
    .await
    .expect("database");

    Response::Success(invite_id)
}

pub async fn invite_wave(
    Query(InviteWaveData { min_level }): Query<InviteWaveData>,
    AuthenticatedUser {
        user: DbUser { level, .. },
        ..
    }: AuthenticatedUser<DbUser>,
    State(AppState { db, roles, .. }): State<AppState>,
) -> Response<u64> {
    if !roles
        .get_current(level)
        .map(|v| v.permissions.wave)
        .unwrap_or(false)
    {
        return Response::Failture(api::Error::Forbidden.into());
    }

    let res = sqlx::query!(
        "UPDATE users SET invites = invites + 1 WHERE level >= ?",
        min_level
    )
    .execute(&db)
    .await
    .expect("database");

    Response::Success(res.rows_affected())
}

pub async fn get_all_roles(
    _: AuthenticatedUser, // NOTE: for all users?
    State(AppState { roles, .. }): State<AppState>,
) -> Response<&'static Vec<UserRole>> {
    Response::Success(&roles.0)
}

pub async fn get_user_role(
    Path(UserIDPath { user_id }): Path<UserIDPath>,
    AuthenticatedUser {
        user: DbUser { level, .. },
        ..
    }: AuthenticatedUser<DbUser>,
    State(AppState { db, roles, .. }): State<AppState>,
) -> Response<&'static UserRole> {
    if !roles
        .get_current(level)
        .map(|v| v.permissions.promote)
        .unwrap_or(false)
    {
        return Response::Failture(api::Error::Forbidden.into());
    }

    let res = sqlx::query!("SELECT level FROM users WHERE id = ?", user_id)
        .fetch_optional(&db)
        .await
        .expect("database");

    match res.and_then(|v| roles.get_current(v.level)) {
        Some(v) => Response::Success(v),
        None => Response::Failture(api::Error::ObjectNotFound.into()),
    }
}

pub async fn promote_user(
    Path(UserIDPath { user_id }): Path<UserIDPath>,
    AuthenticatedUser {
        user: DbUser { level, .. },
        ..
    }: AuthenticatedUser<DbUser>,
    State(AppState { db, roles, .. }): State<AppState>,
    Json(PromoteUserBody { level: to_level }): Json<PromoteUserBody>,
) -> Response<u64> {
    if to_level > level
        && !roles
            .get_current(level)
            .map(|v| v.permissions.promote)
            .unwrap_or(false)
    {
        return Response::Failture(api::Error::Forbidden.into());
    }

    let res = sqlx::query!(
        "UPDATE users SET level = ? WHERE id = ? AND level < ?",
        to_level,
        user_id,
        level
    )
    .execute(&db)
    .await
    .expect("database");

    match res.rows_affected() {
        0 => Response::Failture(
            api::Error::ObjectNotFound.detail("User does not exists or have too big level".into()),
        ),
        v => Response::Success(v),
    }
}

pub async fn get_spaces(
    Query(Paging { page }): Query<Paging>,
    AuthenticatedUser { user, .. }: AuthenticatedUser,
    State(AppState { db, .. }): State<AppState>,
) -> Response<Vec<UserSpaceResponse>> {
    let limit = 50;
    let offset = page * limit;
    let user_id: &str = &user;
    let res = sqlx::query!(
        "SELECT * FROM spaces WHERE owner_id = ? LIMIT ? OFFSET ?",
        user_id,
        limit,
        offset
    )
    .fetch_all(&db)
    .await
    .expect("database");

    Response::Success(
        res.into_iter()
            .map(|v| UserSpaceResponse {
                id: v.id,
                title: v.title,
            })
            .collect(),
    )
}

pub async fn get_user_spaces(
    Query(Paging { page }): Query<Paging>,
    Path(UserIDPath { user_id }): Path<UserIDPath>,
    AuthenticatedUser {
        user: DbUser { level, .. },
        ..
    }: AuthenticatedUser<DbUser>,
    State(AppState { db, roles, .. }): State<AppState>,
) -> Response<Vec<UserSpaceResponse>> {
    if !roles
        .get_current(level)
        .map(|v| v.permissions.spaces_manage)
        .unwrap_or(false)
    {
        return Response::Failture(api::Error::Forbidden.into());
    }

    let limit = 50;
    let offset = page * limit;

    let res = sqlx::query!(
        "SELECT * FROM spaces WHERE owner_id = ? LIMIT ? OFFSET ?",
        user_id,
        limit,
        offset
    )
    .fetch_all(&db)
    .await
    .expect("database");

    Response::Success(
        res.into_iter()
            .map(|v| UserSpaceResponse {
                id: v.id,
                title: v.title,
            })
            .collect(),
    )
}
