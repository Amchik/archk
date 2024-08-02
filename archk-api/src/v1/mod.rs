use std::{any::Any, mem};

use archk::v1::api;
use axum::{
    body::Body,
    extract::Request,
    http::header::CONTENT_TYPE,
    middleware::{self, Next},
    response::{IntoResponse, Response},
    routing::{get, patch, post, put},
    Router,
};
use http_body_util::BodyExt;
use tower::ServiceBuilder;
use tower_http::{catch_panic::CatchPanicLayer, trace::TraceLayer};

use crate::app::AppState;

mod auth;
mod extra;
pub mod routes;
mod space;
mod user;

pub fn get_routes() -> Router<AppState> {
    routes::get_routes()
        // .route("/auth", post(auth::authorize))
        // .route("/users", get(user::get_users))
        // .route("/users/roles", get(user::get_all_roles))
        // .route("/user", get(user::get_self))
        // .route("/user", put(user::register))
        // .route("/user", patch(user::patch_user))
        // .route("/user/spaces", get(user::get_spaces))
        // .route("/user/@:user_id", get(user::get_user))
        // .route("/user/@:user_id", patch(user::reset_user_password))
        // .route("/user/@:user_id/role", get(user::get_user_role))
        // .route("/user/@:user_id/role", patch(user::promote_user))
        // .route("/user/@:user_id/spaces", get(user::get_user_spaces))
        // .route("/user/invites", get(user::get_invites))
        // .route("/user/invites", put(user::create_invite))
        // .route("/user/invites/wave", post(user::invite_wave))
        // .route("/space", put(space::create_space))
        .route(
            "/space/:space_id",
            get(space::get_space)
                .patch(space::patch_space)
                .delete(space::delete_space),
        )
        .route(
            "/space/:space_id/account",
            get(space::get_accounts).put(space::create_account),
        )
        .route(
            "/space/:space_id/account/:acc_id",
            get(space::get_account_by_id)
                .patch(space::patch_account_by_id)
                .delete(space::delete_account_by_id),
        )
        .route(
            "/space/:space_id/account/:acc_id/items",
            get(space::get_items_of_account),
        )
        .route(
            "/space/:space_id/item",
            get(space::get_items).put(space::create_item),
        )
        .route(
            "/space/:space_id/item/:item_id",
            get(space::get_item_by_id)
                .patch(space::patch_item)
                .delete(space::delete_item),
        )
        .fallback(fallback)
        .layer(
            ServiceBuilder::new()
                .layer(TraceLayer::new_for_http())
                .layer(CatchPanicLayer::custom(catch_panic))
                .layer(middleware::from_fn(catch_error)),
        )
}

async fn fallback() -> api::Response {
    api::Response::Failture(api::Error::NoEndpoint.into())
}

async fn catch_error(request: Request, next: Next) -> Response {
    let mut response = next.run(request).await;

    let content_type = response
        .headers()
        .get(CONTENT_TYPE)
        .and_then(|v| v.to_str().ok());

    if content_type
        .map(|v| v != "application/json")
        .unwrap_or(true)
        && (response.status().is_client_error() || response.status().is_server_error())
    {
        let body = mem::replace(response.body_mut(), Body::empty());

        let Ok(detail) = body.collect().await else {
            return api::Response::<api::NeverSerialize>::Failture(
                api::Error::Internal.detail("unable to read response body".into()),
            )
            .into_response();
        };
        let detail = detail.to_bytes();
        let detail = String::from_utf8_lossy(&detail);

        let mut new_response = api::Response::<api::NeverSerialize>::Failture(
            api::Error::ProcessingError.detail(detail.into_owned().into()),
        )
        .into_response();
        *new_response.status_mut() = response.status();

        new_response
    } else {
        response
    }
}

fn catch_panic(_err: Box<dyn Any + Send + 'static>) -> Response {
    api::Response::<api::NeverSerialize>::Failture(api::Error::Internal.into()).into_response()
}
