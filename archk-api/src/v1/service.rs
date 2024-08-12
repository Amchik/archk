use archk::{
    v1::{
        api::{self, Response},
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
    extra::{AuthenticatedUser, DbUser},
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
