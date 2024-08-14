use std::{any::Any, mem};

use archk::v1::api;
use axum::{
    body::Body,
    extract::Request,
    http::header::CONTENT_TYPE,
    middleware::{self, Next},
    response::{IntoResponse, Response},
    routing::{patch, post, put},
    Router,
};
use http_body_util::BodyExt;
use tower::ServiceBuilder;
use tower_http::{catch_panic::CatchPanicLayer, trace::TraceLayer};

use crate::app::AppState;

mod auth;
mod extra;
pub mod routes;
mod service;
mod space;
mod user;

pub fn get_routes() -> Router<AppState> {
    routes::get_routes().fallback(fallback).layer(
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
