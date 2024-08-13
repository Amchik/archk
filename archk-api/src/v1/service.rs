use archk::{
    v1::{
        api::{self, Response},
        auth::{Token, TokenTy},
        service::{ServiceAccount, ServiceAccountID, ServiceAccountTy},
        space::SpaceID,
    },
    Documentation,
};
use axum::{
    extract::{Path, Query, State},
    Json,
};
use serde::{Deserialize, Serialize};

use crate::app::AppState;

use super::{
    extra::{AuthenticatedUser, DbService, DbUser},
    space::SpacePath,
};

#[derive(Deserialize)]
pub struct ServiceFetchOptions {
    #[serde(default)]
    pub page: u32,

    #[serde(default)]
    pub all: bool,
}

#[derive(Deserialize)]
// FIXME: #[derive(Documentation)]
pub struct CreateServiceBody {
    /// Service type.
    pub ty: ServiceAccountTy,
    /// Space ID. Set to `null` to create admin service.
    pub space_id: Option<SpaceID>,
}

#[derive(Deserialize)]
pub struct ServiceAccountPath {
    pub service_account_id: String,
}

#[derive(Serialize, Documentation)]
pub struct ServiceAccountResponse {
    /// Service ID
    pub id: String,
    /// Space ID service belongs to
    pub space_id: Option<String>,
    /// Service type
    pub ty: i64,
}

#[derive(Serialize, Documentation)]
pub struct ServiceTokenResponse {
    /// Bearer token
    pub token: String,
}

pub async fn get_services(
    Query(ServiceFetchOptions { page, all }): Query<ServiceFetchOptions>,
    AuthenticatedUser {
        user: DbUser { level, .. },
        ..
    }: AuthenticatedUser<DbUser>,
    State(AppState { db, roles, .. }): State<AppState>,
) -> Response<Vec<ServiceAccountResponse>> {
    if roles
        .get_current(level)
        .filter(|v| v.permissions.services_manage)
        .is_none()
    {
        return Response::Failture(api::Error::Forbidden.into());
    }

    let (limit, offset) = (50, 50 * page as i64);

    let res = if all {
        sqlx::query_as!(
            ServiceAccountResponse,
            "SELECT * FROM service_accounts LIMIT ? OFFSET ?",
            limit,
            offset
        )
        .fetch_all(&db)
        .await
    } else {
        sqlx::query_as!(
            ServiceAccountResponse,
            "SELECT * FROM service_accounts WHERE space_id IS NULL LIMIT ? OFFSET ?",
            limit,
            offset
        )
        .fetch_all(&db)
        .await
    };

    Response::Success(res.expect("database"))
}

pub async fn get_space_services(
    Path(SpacePath { space_id }): Path<SpacePath>,
    Query(ServiceFetchOptions { page, .. }): Query<ServiceFetchOptions>,
    AuthenticatedUser {
        user: DbUser {
            id: user_id, level, ..
        },
        ..
    }: AuthenticatedUser<DbUser>,
    State(AppState { db, roles, .. }): State<AppState>,
) -> Response<Vec<ServiceAccountResponse>> {
    let is_admin = roles
        .get_current(level)
        .filter(|v| v.permissions.services_manage && v.permissions.spaces_manage)
        .is_some();

    let (limit, offset) = (50, 50 * page as i64);

    let space_id: &str = &space_id;
    let res = if is_admin {
        sqlx::query_as!(
            ServiceAccountResponse,
            "
            SELECT
                service_accounts.id,
                service_accounts.ty,
                service_accounts.space_id
            FROM service_accounts
            WHERE service_accounts.space_id = ?
            LIMIT ? OFFSET ?",
            space_id,
            limit,
            offset
        )
        .fetch_all(&db)
        .await
    } else {
        sqlx::query_as!(
            ServiceAccountResponse,
            "
            SELECT
                service_accounts.id,
                service_accounts.ty,
                service_accounts.space_id
            FROM service_accounts
                INNER JOIN spaces ON
                    service_accounts.space_id = spaces.id
            WHERE service_accounts.space_id = ? AND spaces.owner_id = ?
            LIMIT ? OFFSET ?",
            space_id,
            user_id,
            limit,
            offset
        )
        .fetch_all(&db)
        .await
    };

    Response::Success(res.expect("database"))
}

pub async fn create_service(
    AuthenticatedUser {
        user: DbUser {
            id: user_id, level, ..
        },
        ..
    }: AuthenticatedUser<DbUser>,
    State(AppState { db, roles, .. }): State<AppState>,
    Json(CreateServiceBody { ty, space_id }): Json<CreateServiceBody>,
) -> Response<ServiceAccount> {
    let perms = roles
        .get_current(level)
        .map(|v| &v.permissions)
        .cloned()
        .unwrap_or_default();

    if !perms.services || (ty.is_admin() && !perms.services_manage) {
        return Response::Failture(api::Error::Forbidden.into());
    }

    if space_id.is_none() && ty.is_space_required() {
        return Response::Failture(
            api::Error::MalformedData
                .detail("this service type `ty` requires `space_id` to be specified".into()),
        );
    }

    if let Some(ref space_id) = space_id {
        if !perms.spaces_manage {
            let space_id: &str = &space_id;
            let res = sqlx::query!("SELECT owner_id FROM spaces WHERE id = ?", space_id)
                .fetch_optional(&db)
                .await
                .expect("database")
                .filter(|v| v.owner_id == user_id);

            if res.is_none() {
                return Response::Failture(api::Error::Forbidden.into());
            }
        }
    }

    let id = ServiceAccountID::new();
    let id_str: &str = &id;
    let space_id_ref = space_id.as_deref();
    let ty_idx: i64 = ty.into();

    let res = sqlx::query!(
        "INSERT INTO service_accounts(id, ty, space_id) VALUES (?, ?, ?)",
        id_str,
        ty_idx,
        space_id_ref
    )
    .execute(&db)
    .await;

    match res {
        Err(sqlx::Error::Database(e)) if e.is_foreign_key_violation() => {
            Response::Failture(api::Error::ObjectNotFound.into())
        }
        Ok(_) => Response::Success(ServiceAccount { id, ty, space_id }),
        Err(e) => panic!("database error: {e}"),
    }
}

pub async fn delete_service(
    Path(ServiceAccountPath { service_account_id }): Path<ServiceAccountPath>,
    AuthenticatedUser {
        user: DbUser {
            id: user_id, level, ..
        },
        ..
    }: AuthenticatedUser<DbUser>,
    State(AppState { db, roles, .. }): State<AppState>,
) -> Response<u64> {
    if roles
        .get_current(level)
        .filter(|v| v.permissions.services_manage)
        .is_none()
    {
        let res = sqlx::query!(
            "
            SELECT spaces.owner_id
            FROM service_accounts
                INNER JOIN spaces ON spaces.id = service_accounts.space_id
            WHERE service_accounts.id = ?",
            service_account_id
        )
        .fetch_optional(&db)
        .await
        .expect("database")
        .filter(|v| v.owner_id == user_id);

        if res.is_none() {
            return Response::Failture(api::Error::ObjectNotFound.into());
        }
    }

    let res = sqlx::query!(
        "DELETE FROM service_accounts WHERE id = ?",
        service_account_id
    )
    .execute(&db)
    .await
    .expect("database")
    .rows_affected();

    if res == 0 {
        Response::Failture(api::Error::ObjectNotFound.into())
    } else {
        Response::Success(res)
    }
}

pub async fn get_tokens(
    Path(ServiceAccountPath { service_account_id }): Path<ServiceAccountPath>,
    AuthenticatedUser {
        user: DbUser {
            id: user_id, level, ..
        },
        ..
    }: AuthenticatedUser<DbUser>,
    State(AppState { db, roles, .. }): State<AppState>,
) -> Response<i32> {
    let (permission_services, permission_services_manage) = roles
        .get_current(level)
        .map(|v| (v.permissions.services, v.permissions.services_manage))
        .unwrap_or_default();

    if !permission_services {
        return Response::Failture(api::Error::Forbidden.into());
    }

    let (is_none, count, owner_id) = sqlx::query!(
        "
        SELECT
            (service_accounts.id IS NULL) AS is_none,
            COUNT(service_tokens.service_id) as count,
            spaces.owner_id AS owner_id
        FROM service_accounts
            INNER JOIN service_tokens
                ON service_tokens.service_id = service_accounts.id
            LEFT JOIN spaces
                ON spaces.id = service_accounts.space_id
        WHERE service_accounts.id = ?
        ",
        service_account_id
    )
    .fetch_one(&db)
    .await
    .map(|v| (v.is_none == 1, v.count, v.owner_id))
    .expect("database");

    if is_none || (owner_id != Some(user_id) && !permission_services_manage) {
        return Response::Failture(api::Error::ObjectNotFound.into());
    }

    Response::Success(count)
}

pub async fn put_token(
    Path(ServiceAccountPath { service_account_id }): Path<ServiceAccountPath>,
    AuthenticatedUser {
        user: DbUser {
            id: user_id, level, ..
        },
        ..
    }: AuthenticatedUser<DbUser>,
    State(AppState { db, roles, .. }): State<AppState>,
) -> Response<ServiceTokenResponse> {
    let services_manage = roles
        .get_current(level)
        .filter(|v| v.permissions.services_manage)
        .is_some();

    let res = sqlx::query!(
        "SELECT spaces.owner_id
        FROM service_accounts
            LEFT JOIN spaces ON spaces.id = service_accounts.space_id
        WHERE service_accounts.id = ?",
        service_account_id
    )
    .fetch_optional(&db)
    .await
    .expect("database")
    .filter(|v| services_manage || v.owner_id == Some(user_id));

    // check is service exists and user have permission to use them
    if res.is_none() {
        return Response::Failture(api::Error::ObjectNotFound.into());
    }

    let token = Token::new(TokenTy::Service);
    let iat = token.iat as i64;
    let rnd = token.rnd as i64;

    let res = sqlx::query!(
        "INSERT INTO service_tokens(iat, rnd, service_id) VALUES (?, ?, ?)",
        iat,
        rnd,
        service_account_id
    )
    .execute(&db)
    .await;

    match res {
        Ok(_) => Response::Success(ServiceTokenResponse {
            token: token.to_string(),
        }),
        Err(sqlx::Error::Database(e)) if e.is_foreign_key_violation() => {
            tracing::warn!(
                service_id = service_account_id,
                "Database returned unexpected foreign key violation"
            );
            Response::Failture(api::Error::ObjectNotFound.into())
        }
        Err(e) => panic!("database error: {e}"),
    }
}

pub async fn revoke_all_tokens(
    Path(ServiceAccountPath { service_account_id }): Path<ServiceAccountPath>,
    AuthenticatedUser {
        user: DbUser {
            id: user_id, level, ..
        },
        ..
    }: AuthenticatedUser<DbUser>,
    State(AppState { db, roles, .. }): State<AppState>,
) -> Response<u64> {
    if roles
        .get_current(level)
        .filter(|v| v.permissions.services_manage)
        .is_none()
    {
        let res = sqlx::query!(
            "
            SELECT spaces.owner_id
            FROM service_accounts
                INNER JOIN spaces ON spaces.id = service_accounts.space_id
            WHERE service_accounts.id = ?",
            service_account_id
        )
        .fetch_optional(&db)
        .await
        .expect("database")
        .filter(|v| v.owner_id == user_id);

        if res.is_none() {
            return Response::Failture(api::Error::ObjectNotFound.into());
        }
    }

    let res = sqlx::query!(
        "DELETE FROM service_tokens WHERE service_id = ?",
        service_account_id
    )
    .execute(&db)
    .await
    .expect("database");

    Response::Success(res.rows_affected())
}

pub mod ssh {
    use archk::v1::user::ssh::SSHKeyTy;

    use super::*;

    #[derive(Deserialize, Documentation)]
    pub struct FingerprintBody {
        /// SSH key fingerprint in base64 without any prefixes (like `SHA256:`)
        pub fingerprint: String,
    }

    #[derive(Serialize, Documentation)]
    pub struct SSHKeyResponse {
        /// Full public key string with key type
        pub public_key: String,
        /// ID of key owner (can be used as key comment)
        pub user_id: String,
    }

    pub async fn fetch_ssh_keys_by_fingerprint(
        AuthenticatedUser {
            user: DbService { ty, .. },
            ..
        }: AuthenticatedUser<DbService>,
        State(AppState { db, .. }): State<AppState>,
        Json(FingerprintBody { fingerprint }): Json<FingerprintBody>,
    ) -> Response<Vec<SSHKeyResponse>> {
        if ty != ServiceAccountTy::SSHAuthority {
            return Response::Failture(api::Error::Forbidden.into());
        }

        let res = sqlx::query!(
            "SELECT pubkey_ty, pubkey_val, owner_id
            FROM users_ssh_keys
            WHERE pubkey_fingerprint = ?",
            fingerprint
        )
        .fetch_all(&db)
        .await
        .expect("database");

        if res.is_empty() {
            Response::Failture(api::Error::ObjectNotFound.into())
        } else {
            Response::Success(
                res.into_iter()
                    .flat_map(|v| {
                        Some(SSHKeyResponse {
                            public_key: format!(
                                "{} {}",
                                SSHKeyTy::try_from(v.pubkey_ty)
                                    .map(Into::<&'static str>::into)
                                    .ok()?,
                                v.pubkey_val
                            ),
                            user_id: v.owner_id,
                        })
                    })
                    .collect(),
            )
        }
    }
}
