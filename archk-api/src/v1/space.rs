use archk::{
    v1::{
        api::{self, Response},
        models::MayIgnored,
        space::{Space, SpaceAccount, SpaceID, SpaceItem, SpaceItemID, SpaceItemTy},
        user::{User, UserID},
    },
    Documentation,
};
use axum::{
    extract::{Path, Query, State},
    Json,
};
use serde::{Deserialize, Serialize};

use crate::app::AppState;

use super::extra::{AuthenticatedUser, DbUser};

#[derive(Deserialize)]
pub struct SpacePath {
    pub space_id: SpaceID,
}
#[derive(Deserialize)]
pub struct SpaceAccountPath {
    pub space_id: SpaceID,
    pub acc_id: String,
}
#[derive(Deserialize)]
pub struct SpaceItemPath {
    pub space_id: SpaceID,
    pub item_id: String,
}

#[derive(Deserialize)]
pub struct PatchSpace {
    pub title: String,
}

#[derive(Deserialize)]
pub struct Paging {
    #[serde(default)]
    pub page: u32,
}

#[derive(Deserialize)]
pub struct PatchAccountBody {
    #[serde(default, skip_serializing_if = "MayIgnored::is_ignored")]
    pub pl_name: MayIgnored<Option<String>>,
    #[serde(default, skip_serializing_if = "MayIgnored::is_ignored")]
    pub pl_displayname: MayIgnored<Option<String>>,
}
#[derive(Deserialize)]
pub struct PatchItemBody {
    #[serde(default, skip_serializing_if = "MayIgnored::is_ignored")]
    pub title: MayIgnored<String>,
}

#[derive(Deserialize)]
pub struct CreateSpaceItemBody {
    pub title: String,
    #[serde(default)]
    pub ty: SpaceItemTy,
    pub pl_serial: String,
    #[serde(default)]
    pub owner_id: Option<String>,
}

#[derive(Serialize)]
pub struct GetSpaceResponse {
    pub space: Space,
    pub owner: User,
}
#[derive(Serialize, Deserialize)]
pub struct SpaceAccountWithoutSpaceID {
    pub pl_id: String,
    pub pl_name: Option<String>,
    pub pl_displayname: Option<String>,
}
#[derive(Serialize)]
pub struct SpaceItemWithoutSpaceID {
    pub id: String,
    pub title: String,
    pub ty: i64,
    pub pl_serial: String,
    pub owner_id: Option<String>,
}
#[derive(Serialize)]
pub struct GetSpaceItemResponse {
    pub item: SpaceItemWithoutSpaceID,
    pub owner: Option<SpaceAccountWithoutSpaceID>,
}

#[derive(Serialize, Documentation)]
pub struct SpaceItemLogEntry {
    /// Global space log ID
    pub id: String,
    /// Creation timestamp
    pub created_at: i64,

    /// Action
    pub act: i64,
    /// Account platform ID if any
    pub sp_acc_id: Option<String>,
    /// Item ID if any
    pub sp_item_id: Option<String>,
}

pub async fn create_space(
    AuthenticatedUser {
        user: DbUser {
            id: user_id, level, ..
        },
        ..
    }: AuthenticatedUser<DbUser>,
    State(AppState { db, roles, .. }): State<AppState>,
    Json(PatchSpace { title }): Json<PatchSpace>,
) -> Response<Space> {
    let can_create_spaces = roles
        .get_current(level)
        .map(|v| v.permissions.spaces)
        .unwrap_or(false);

    if !can_create_spaces {
        return Response::Failture(api::Error::Forbidden.into());
    }

    let space_id = SpaceID::new();
    let id: &str = &space_id;
    let _ = sqlx::query!(
        "INSERT INTO spaces(id, title, owner_id) VALUES (?, ?, ?)",
        id,
        title,
        user_id
    )
    .execute(&db)
    .await
    .expect("database");

    Response::Success(Space {
        id: space_id,
        title,
        owner_id: UserID::from(user_id).expect("user id from database"),
    })
}

pub async fn get_space(
    Path(SpacePath { space_id }): Path<SpacePath>,
    AuthenticatedUser {
        user: DbUser {
            id: user_id, level, ..
        },
        ..
    }: AuthenticatedUser<DbUser>,
    State(AppState { db, roles, .. }): State<AppState>,
) -> Response<GetSpaceResponse> {
    let can_manage_spaces = roles
        .get_current(level)
        .map(|v| v.permissions.spaces_manage)
        .unwrap_or(false);

    let space_id: &str = &space_id;
    let res = sqlx::query!(
        r#"
        SELECT
            spaces.id as sp_id,
            spaces.title as sp_title,
            spaces.owner_id as user_id,
            users.name as user_name,
            users.invited_by as user_invited_by
        FROM spaces
            INNER JOIN users ON spaces.owner_id = users.id
        WHERE spaces.id = ?
    "#,
        space_id
    )
    .fetch_optional(&db)
    .await
    .expect("database");

    match res {
        Some(res) if !can_manage_spaces && res.user_id != user_id => {
            Response::Failture(api::Error::ObjectNotFound.into())
        }

        None => Response::Failture(api::Error::ObjectNotFound.into()),

        Some(res) => {
            let user_id = UserID::from(res.user_id).unwrap();

            Response::Success(GetSpaceResponse {
                space: Space {
                    id: SpaceID::from(res.sp_id).unwrap(),
                    title: res.sp_title,
                    owner_id: user_id.clone(),
                },
                owner: User {
                    id: user_id,
                    name: res.user_name,
                    invited_by: res.user_invited_by,
                },
            })
        }
    }
}

pub async fn patch_space(
    Path(SpacePath { space_id }): Path<SpacePath>,
    AuthenticatedUser {
        user: DbUser {
            id: user_id, level, ..
        },
        ..
    }: AuthenticatedUser<DbUser>,
    State(AppState { db, roles, .. }): State<AppState>,
    Json(PatchSpace { title }): Json<PatchSpace>,
) -> Response<u64> {
    let can_manage_spaces = roles
        .get_current(level)
        .map(|v| v.permissions.spaces_manage)
        .unwrap_or(false);

    let space_id: &str = &space_id;
    let stmt = if can_manage_spaces {
        sqlx::query!("UPDATE spaces SET title = ? WHERE id = ?", title, space_id)
    } else {
        sqlx::query!(
            "UPDATE spaces SET title = ? WHERE id = ? AND owner_id = ?",
            title,
            space_id,
            user_id
        )
    };

    let res = stmt.execute(&db).await.expect("database").rows_affected();

    if res == 0 {
        Response::Failture(api::Error::ObjectNotFound.into())
    } else {
        Response::Success(res)
    }
}

pub async fn delete_space(
    Path(SpacePath { space_id }): Path<SpacePath>,
    AuthenticatedUser {
        user: DbUser {
            id: user_id, level, ..
        },
        ..
    }: AuthenticatedUser<DbUser>,
    State(AppState { db, roles, .. }): State<AppState>,
) -> Response<u64> {
    let can_manage_spaces = roles
        .get_current(level)
        .map(|v| v.permissions.spaces_manage)
        .unwrap_or(false);

    let space_id: &str = &space_id;
    let stmt = if can_manage_spaces {
        sqlx::query!("DELETE FROM spaces WHERE id = ?", space_id)
    } else {
        sqlx::query!(
            "DELETE FROM spaces WHERE id = ? AND owner_id = ?",
            space_id,
            user_id
        )
    };

    let res = stmt.execute(&db).await.expect("database").rows_affected();

    if res == 0 {
        Response::Failture(api::Error::ObjectNotFound.into())
    } else {
        Response::Success(res)
    }
}

pub async fn get_accounts(
    Path(SpacePath { space_id }): Path<SpacePath>,
    Query(Paging { page }): Query<Paging>,
    AuthenticatedUser {
        user: DbUser {
            id: user_id, level, ..
        },
        ..
    }: AuthenticatedUser<DbUser>,
    State(AppState { db, roles, .. }): State<AppState>,
) -> Response<Vec<SpaceAccountWithoutSpaceID>> {
    let can_manage_spaces = roles
        .get_current(level)
        .map(|v| v.permissions.spaces_manage)
        .unwrap_or(false);

    let space_id: &str = &space_id;
    let limit = 50;
    let offset = (page as i64) * limit;
    let stmt = if can_manage_spaces {
        sqlx::query_as!(
            SpaceAccountWithoutSpaceID,
            "SELECT pl_id, pl_name, pl_displayname FROM spaces_accounts WHERE space_id = ? LIMIT ? OFFSET ?",
            space_id, limit, offset
        )
        .fetch_all(&db)
        .await
    } else {
        sqlx::query_as!(
            SpaceAccountWithoutSpaceID,
            r#"SELECT pl_id, pl_name, pl_displayname
            FROM spaces_accounts
                INNER JOIN spaces ON spaces.id = spaces_accounts.space_id
            WHERE
                spaces_accounts.space_id = ? AND spaces.owner_id = ?
            LIMIT ? OFFSET ?"#,
            space_id,
            user_id,
            limit,
            offset
        )
        .fetch_all(&db)
        .await
    };

    let res = stmt.expect("database");

    Response::Success(res)
}

pub async fn create_account(
    Path(SpacePath { space_id }): Path<SpacePath>,
    AuthenticatedUser {
        user: DbUser {
            id: user_id, level, ..
        },
        ..
    }: AuthenticatedUser<DbUser>,
    State(AppState { db, roles, .. }): State<AppState>,
    Json(SpaceAccountWithoutSpaceID {
        pl_id,
        pl_name,
        pl_displayname,
    }): Json<SpaceAccountWithoutSpaceID>,
) -> Response<SpaceAccount> {
    let can_manage_spaces = roles
        .get_current(level)
        .map(|v| v.permissions.spaces_manage)
        .unwrap_or(false);

    let space_id_str: &str = &space_id;
    if !can_manage_spaces {
        // TODO: via one query if possible
        let res = sqlx::query!("SELECT owner_id FROM spaces WHERE id = ?", space_id_str)
            .fetch_optional(&db)
            .await
            .expect("database")
            .map(|v| v.owner_id);
        if res != Some(user_id) {
            return Response::Failture(api::Error::ObjectNotFound.into());
        }
    }

    let res = sqlx::query!(
        "INSERT INTO spaces_accounts(pl_id, space_id, pl_name, pl_displayname) VALUES (?, ?, ?, ?)",
        pl_id,
        space_id_str,
        pl_name,
        pl_displayname
    )
    .execute(&db)
    .await;

    match res {
        Ok(_) => Response::Success(SpaceAccount {
            pl_id,
            pl_name,
            pl_displayname,
            space_id,
        }),
        Err(sqlx::Error::Database(err)) if err.is_foreign_key_violation() => {
            Response::Failture(api::Error::ObjectNotFound.into())
        }
        Err(sqlx::Error::Database(err)) if err.is_unique_violation() => Response::Failture(
            api::Error::Conflict.detail("account with given `pl_id` already exists".into()),
        ),
        Err(e) => panic!("database error: {e}"),
    }
}

pub async fn get_account_by_id(
    Path(SpaceAccountPath { space_id, acc_id }): Path<SpaceAccountPath>,
    AuthenticatedUser {
        user: DbUser {
            id: user_id, level, ..
        },
        ..
    }: AuthenticatedUser<DbUser>,
    State(AppState { db, roles, .. }): State<AppState>,
) -> Response<SpaceAccount> {
    let can_manage_spaces = roles
        .get_current(level)
        .map(|v| v.permissions.spaces_manage)
        .unwrap_or(false);

    let space_id_ref: &str = &space_id;
    let res = sqlx::query!(
        r#"
        SELECT spaces_accounts.*, spaces.owner_id as space_owner_id
        FROM spaces_accounts
            INNER JOIN spaces ON spaces.id = spaces_accounts.space_id
        WHERE space_id = ? AND pl_id = ?"#,
        space_id_ref,
        acc_id
    )
    .fetch_optional(&db)
    .await
    .expect("database");

    match res {
        Some(v) if can_manage_spaces || v.space_owner_id == user_id => {
            Response::Success(SpaceAccount {
                pl_id: v.pl_id,
                space_id,
                pl_name: v.pl_name,
                pl_displayname: v.pl_displayname,
            })
        }
        _ => Response::Failture(api::Error::ObjectNotFound.into()),
    }
}

pub async fn patch_account_by_id(
    Path(SpaceAccountPath { space_id, acc_id }): Path<SpaceAccountPath>,
    AuthenticatedUser {
        user: DbUser {
            id: user_id, level, ..
        },
        ..
    }: AuthenticatedUser<DbUser>,
    State(AppState { db, roles, .. }): State<AppState>,
    Json(PatchAccountBody {
        pl_name,
        pl_displayname,
    }): Json<PatchAccountBody>,
) -> Response<u64> {
    if pl_name.is_ignored() && pl_displayname.is_ignored() {
        return Response::Failture(
            api::Error::MalformedData.detail("Expected at least one subject to change".into()),
        );
    }

    let can_manage_spaces = roles
        .get_current(level)
        .map(|v| v.permissions.spaces_manage)
        .unwrap_or(false);

    if !can_manage_spaces {
        let space_id: &str = &space_id;
        let res = sqlx::query!("SELECT owner_id FROM spaces WHERE id = ?", space_id)
            .fetch_optional(&db)
            .await
            .expect("database")
            .map(|v| v.owner_id);

        match res {
            Some(owner_id) if owner_id == user_id => (),
            _ => return Response::Failture(api::Error::ObjectNotFound.into()),
        }
    }

    let mut stmt = String::from("UPDATE spaces_accounts SET ");
    let mut params = Vec::with_capacity(2);

    if let MayIgnored::Value(pl_name) = pl_name {
        stmt.push_str("pl_name = ? ");
        params.push(pl_name);
    }
    if let MayIgnored::Value(pl_displayname) = pl_displayname {
        stmt.push_str("pl_displayname = ? ");
        params.push(pl_displayname);
    }

    stmt.push_str("WHERE pl_id = ? AND space_id = ?");
    params.push(Some(acc_id));
    params.push(Some(space_id.into()));

    let mut res: sqlx::query::Query<sqlx::Sqlite, _> = sqlx::query(&stmt);
    for param in params {
        res = res.bind(param);
    }

    let res = res.execute(&db).await.expect("database").rows_affected();

    if res != 0 {
        Response::Success(res)
    } else {
        Response::Failture(api::Error::ObjectNotFound.into())
    }
}

pub async fn delete_account_by_id(
    Path(SpaceAccountPath { space_id, acc_id }): Path<SpaceAccountPath>,
    AuthenticatedUser {
        user: DbUser {
            id: user_id, level, ..
        },
        ..
    }: AuthenticatedUser<DbUser>,
    State(AppState { db, roles, .. }): State<AppState>,
) -> Response<u64> {
    let can_manage_spaces = roles
        .get_current(level)
        .map(|v| v.permissions.spaces_manage)
        .unwrap_or(false);

    if !can_manage_spaces {
        let space_id: &str = &space_id;
        let res = sqlx::query!("SELECT owner_id FROM spaces WHERE id = ?", space_id)
            .fetch_optional(&db)
            .await
            .expect("database")
            .map(|v| v.owner_id);

        match res {
            Some(owner_id) if owner_id == user_id => (),
            _ => return Response::Failture(api::Error::ObjectNotFound.into()),
        }
    }

    let space_id: &str = &space_id;
    let res = sqlx::query!(
        r#"UPDATE spaces_logs SET sp_acc_id = NULL WHERE sp_acc_id = ? AND space_id = ?;
        DELETE FROM spaces_accounts WHERE pl_id = ? AND space_id = ?"#,
        acc_id,
        space_id,
        acc_id,
        space_id,
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

pub async fn get_items(
    Path(SpacePath { space_id }): Path<SpacePath>,
    Query(Paging { page }): Query<Paging>,
    AuthenticatedUser {
        user: DbUser {
            id: user_id, level, ..
        },
        ..
    }: AuthenticatedUser<DbUser>,
    State(AppState { db, roles, .. }): State<AppState>,
) -> Response<Vec<SpaceItemWithoutSpaceID>> {
    let can_manage_spaces = roles
        .get_current(level)
        .map(|v| v.permissions.spaces_manage)
        .unwrap_or(false);

    let space_id: &str = &space_id;
    let limit = 50;
    let offset = (page as i64) * limit;
    let stmt = if can_manage_spaces {
        sqlx::query_as!(
            SpaceItemWithoutSpaceID,
        "SELECT id, title, ty, pl_serial, owner_id FROM spaces_items WHERE space_id = ? LIMIT ? OFFSET ?",
        space_id, limit, offset
    )
    .fetch_all(&db)
    .await
    } else {
        sqlx::query_as!(
            SpaceItemWithoutSpaceID,
            r#"
        SELECT
            spaces_items.id,
            spaces_items.title,
            spaces_items.ty,
            spaces_items.pl_serial,
            spaces_items.owner_id
        FROM spaces_items
            INNER JOIN spaces ON spaces.id = spaces_items.space_id
        WHERE
            spaces_items.space_id = ? AND spaces.owner_id = ?
        LIMIT ? OFFSET ?"#,
            space_id,
            user_id,
            limit,
            offset
        )
        .fetch_all(&db)
        .await
    };

    let res = stmt.expect("database");

    Response::Success(res)
}

pub async fn get_items_of_account(
    Path(SpaceAccountPath { space_id, acc_id }): Path<SpaceAccountPath>,
    Query(Paging { page }): Query<Paging>,
    AuthenticatedUser {
        user: DbUser {
            id: user_id, level, ..
        },
        ..
    }: AuthenticatedUser<DbUser>,
    State(AppState { db, roles, .. }): State<AppState>,
) -> Response<Vec<SpaceItemWithoutSpaceID>> {
    let can_manage_spaces = roles
        .get_current(level)
        .map(|v| v.permissions.spaces_manage)
        .unwrap_or(false);

    let space_id: &str = &space_id;
    let limit = 50;
    let offset = (page as i64) * limit;
    let stmt = if can_manage_spaces {
        sqlx::query_as!(
            SpaceItemWithoutSpaceID,
        "SELECT id, title, ty, pl_serial, owner_id FROM spaces_items WHERE space_id = ? AND owner_id = ? LIMIT ? OFFSET ?",
        space_id, acc_id, limit, offset
    )
    .fetch_all(&db)
    .await
    } else {
        sqlx::query_as!(
            SpaceItemWithoutSpaceID,
            r#"
        SELECT
            spaces_items.id,
            spaces_items.title,
            spaces_items.ty,
            spaces_items.pl_serial,
            spaces_items.owner_id
        FROM spaces_items
            INNER JOIN spaces ON spaces.id = spaces_items.space_id
        WHERE
            spaces_items.space_id = ? AND spaces.owner_id = ? AND spaces_items.owner_id = ?
        LIMIT ? OFFSET ?"#,
            space_id,
            user_id,
            acc_id,
            limit,
            offset
        )
        .fetch_all(&db)
        .await
    };

    let res = stmt.expect("database");

    Response::Success(res)
}

pub async fn create_item(
    Path(SpacePath { space_id }): Path<SpacePath>,
    AuthenticatedUser {
        user: DbUser {
            id: user_id, level, ..
        },
        ..
    }: AuthenticatedUser<DbUser>,
    State(AppState { db, roles, .. }): State<AppState>,
    Json(CreateSpaceItemBody {
        title,
        ty,
        pl_serial,
        owner_id,
    }): Json<CreateSpaceItemBody>,
) -> Response<SpaceItem> {
    if owner_id.is_none() && ty.is_owner_required() {
        return Response::Failture(api::Error::MalformedData.detail(
            format!("item type `ty` ({ty}) should belong to their owner but `owner_id` isn't specified or null").into(),
        ));
    }

    let can_manage_spaces = roles
        .get_current(level)
        .map(|v| v.permissions.spaces_manage)
        .unwrap_or(false);

    let space_id_str: &str = &space_id;
    if !can_manage_spaces {
        // TODO: via one query if possible
        let res = sqlx::query!("SELECT owner_id FROM spaces WHERE id = ?", space_id_str)
            .fetch_optional(&db)
            .await
            .expect("database")
            .map(|v| v.owner_id);
        if res != Some(user_id) {
            return Response::Failture(api::Error::ObjectNotFound.into());
        }
    }

    let id = SpaceItemID::new();
    let id_str = &id as &str;
    let ty_no: i64 = ty.into();

    let res = sqlx::query!(
        r#"
        INSERT INTO spaces_items(id, title, ty, pl_serial, owner_id, space_id)
        VALUES (?, ?, ?, ?, ?, ?)
        "#,
        id_str,
        title,
        ty_no,
        pl_serial,
        owner_id,
        space_id_str
    )
    .execute(&db)
    .await;

    match res {
        Ok(_) => Response::Success(SpaceItem {
            id,
            title,
            ty,
            pl_serial,
            owner_id,
            space_id,
        }),
        Err(sqlx::Error::Database(err)) if err.is_foreign_key_violation() => Response::Failture(
            api::Error::ObjectNotFound
                .detail("account with specified `owner_id` does not exists".into()),
        ),
        Err(sqlx::Error::Database(err)) if err.is_unique_violation() => Response::Failture(
            api::Error::Conflict.detail("item with that `pl_serial` already exists".into()),
        ),
        Err(e) => panic!("database: {e}"),
    }
}

pub async fn get_item_by_id(
    Path(SpaceItemPath { space_id, item_id }): Path<SpaceItemPath>,
    AuthenticatedUser {
        user: DbUser {
            id: user_id, level, ..
        },
        ..
    }: AuthenticatedUser<DbUser>,
    State(AppState { db, roles, .. }): State<AppState>,
) -> Response<GetSpaceItemResponse> {
    let can_manage_spaces = roles
        .get_current(level)
        .map(|v| v.permissions.spaces_manage)
        .unwrap_or(false);

    let space_id: &str = &space_id;
    let res = sqlx::query!(
        r#"
        SELECT
            spaces_items.id,
            spaces_items.title,
            spaces_items.ty,
            spaces_items.pl_serial,
            spaces_items.owner_id,
            spaces_accounts.pl_name,
            spaces_accounts.pl_displayname,
            spaces.owner_id as space_owner_id
        FROM spaces_items
            LEFT JOIN spaces_accounts
                ON spaces_accounts.pl_id = spaces_items.owner_id
                    AND spaces_accounts.space_id = spaces_items.space_id
            INNER JOIN spaces
                ON spaces.id = spaces_items.space_id
        WHERE spaces_items.space_id = ? AND spaces_items.id = ?
        "#,
        space_id,
        item_id
    )
    .fetch_optional(&db)
    .await
    .expect("database")
    .filter(|v| can_manage_spaces || v.space_owner_id == user_id);

    let Some(res) = res else {
        return Response::Failture(api::Error::ObjectNotFound.into());
    };

    Response::Success(GetSpaceItemResponse {
        item: SpaceItemWithoutSpaceID {
            id: res.id,
            title: res.title,
            ty: res.ty,
            pl_serial: res.pl_serial,
            owner_id: res.owner_id.clone(),
        },
        owner: res.owner_id.map(|v| SpaceAccountWithoutSpaceID {
            pl_id: v,
            pl_name: res.pl_name,
            pl_displayname: res.pl_displayname,
        }),
    })
}

pub async fn patch_item(
    Path(SpaceItemPath { space_id, item_id }): Path<SpaceItemPath>,
    AuthenticatedUser {
        user: DbUser {
            id: user_id, level, ..
        },
        ..
    }: AuthenticatedUser<DbUser>,
    State(AppState { db, roles, .. }): State<AppState>,
    Json(PatchItemBody { title }): Json<PatchItemBody>,
) -> Response<u64> {
    if title.is_ignored() {
        return Response::Failture(
            api::Error::MalformedData.detail("expected at least one subject to change".into()),
        );
    }

    let MayIgnored::Value(title) = title else {
        unreachable!("`title` is checked that it's not ignored");
    };

    let can_manage_spaces = roles
        .get_current(level)
        .map(|v| v.permissions.spaces_manage)
        .unwrap_or(false);

    let space_id: &str = &space_id;
    if !can_manage_spaces {
        let res = sqlx::query!("SELECT owner_id FROM spaces WHERE id = ?", space_id)
            .fetch_optional(&db)
            .await
            .expect("database");

        if res.filter(|v| v.owner_id == user_id).is_none() {
            return Response::Failture(api::Error::ObjectNotFound.into());
        }
    }

    let res = sqlx::query!(
        "UPDATE spaces_items SET title = ? WHERE id = ?",
        title,
        item_id
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

pub async fn delete_item(
    Path(SpaceItemPath { space_id, item_id }): Path<SpaceItemPath>,
    AuthenticatedUser {
        user: DbUser {
            id: user_id, level, ..
        },
        ..
    }: AuthenticatedUser<DbUser>,
    State(AppState { db, roles, .. }): State<AppState>,
) -> Response<u64> {
    let can_manage_spaces = roles
        .get_current(level)
        .map(|v| v.permissions.spaces_manage)
        .unwrap_or(false);

    if !can_manage_spaces {
        let space_id: &str = &space_id;
        let res = sqlx::query!("SELECT owner_id FROM spaces WHERE id = ?", space_id)
            .fetch_optional(&db)
            .await
            .expect("database")
            .map(|v| v.owner_id);

        match res {
            Some(owner_id) if owner_id == user_id => (),
            _ => return Response::Failture(api::Error::ObjectNotFound.into()),
        }
    }

    let space_id: &str = &space_id;
    let res = sqlx::query!(
        r#"UPDATE spaces_logs SET sp_item_id = NULL WHERE sp_item_id = ? AND space_id = ?;
        DELETE FROM spaces_items WHERE id = ? AND space_id = ?"#,
        item_id,
        space_id,
        item_id,
        space_id,
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

pub async fn get_logs_by_account(
    Path(SpaceAccountPath { space_id, acc_id }): Path<SpaceAccountPath>,
    Query(Paging { page }): Query<Paging>,
    AuthenticatedUser {
        user: DbUser {
            id: user_id, level, ..
        },
        ..
    }: AuthenticatedUser<DbUser>,
    State(AppState { db, roles, .. }): State<AppState>,
) -> Response<Vec<SpaceItemLogEntry>> {
    let can_manage_spaces = roles
        .get_current(level)
        .map(|v| v.permissions.spaces_manage)
        .unwrap_or(false);

    let space_id: &str = &space_id;
    if !can_manage_spaces {
        let res = sqlx::query!("SELECT owner_id FROM spaces WHERE id = ?", space_id)
            .fetch_optional(&db)
            .await
            .expect("database")
            .map(|v| v.owner_id);

        match res {
            Some(owner_id) if owner_id == user_id => (),
            _ => return Response::Failture(api::Error::Forbidden.into()),
        }
    }

    let (limit, offset) = (50, 50 * page as i64);
    let res = sqlx::query_as!(
        SpaceItemLogEntry,
        "SELECT id, created_at, act, sp_acc_id, sp_item_id
        FROM spaces_logs
        WHERE sp_acc_id = ? AND space_id = ?
        LIMIT ? OFFSET ?",
        acc_id,
        space_id,
        limit,
        offset
    )
    .fetch_all(&db)
    .await
    .expect("database");

    Response::Success(res)
}

pub async fn get_logs_by_item(
    Path(SpaceItemPath { space_id, item_id }): Path<SpaceItemPath>,
    Query(Paging { page }): Query<Paging>,
    AuthenticatedUser {
        user: DbUser {
            id: user_id, level, ..
        },
        ..
    }: AuthenticatedUser<DbUser>,
    State(AppState { db, roles, .. }): State<AppState>,
) -> Response<Vec<SpaceItemLogEntry>> {
    let can_manage_spaces = roles
        .get_current(level)
        .map(|v| v.permissions.spaces_manage)
        .unwrap_or(false);

    let space_id: &str = &space_id;
    if !can_manage_spaces {
        let res = sqlx::query!("SELECT owner_id FROM spaces WHERE id = ?", space_id)
            .fetch_optional(&db)
            .await
            .expect("database")
            .map(|v| v.owner_id);

        match res {
            Some(owner_id) if owner_id == user_id => (),
            _ => return Response::Failture(api::Error::Forbidden.into()),
        }
    }

    let (limit, offset) = (50, 50 * page as i64);
    let res = sqlx::query_as!(
        SpaceItemLogEntry,
        "SELECT id, created_at, act, sp_acc_id, sp_item_id
        FROM spaces_logs
        WHERE sp_item_id = ? AND space_id = ?
        LIMIT ? OFFSET ?",
        item_id,
        space_id,
        limit,
        offset
    )
    .fetch_all(&db)
    .await
    .expect("database");

    Response::Success(res)
}
