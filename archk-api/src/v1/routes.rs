use axum::{routing::get, Router};

use crate::app::AppState;
use archk::v1::docs;

macro_rules! routes {
    (@method GET $handler:path) => { get($handler) };
    (@method POST $handler:path) => { post($handler) };
    (@method PUT $handler:path) => { put($handler) };
    (@method PATCH $handler:path) => { patch($handler) };
    (@method DELETE $handler:path) => { delete($handler) };
    ( $( $(#[doc = $d:literal])* $method:ident $path:literal => $handler:path $( : $( body($body:path) )? $( res($res:path) )? )? ),* $(,)? ) => {
        /// Get [`axum::Router`] to all endpoints without any fallback or layer.
        /// Use `v1::get_routes()` to include services and fallback
        // $(
        //     #[doc = concat!("# ", stringify!($method), " `", $path, "`")]
        //     $( #[doc = $d] )*
        // )*
        pub fn get_routes() -> Router<AppState> {
            Router::new()
                $( .route($path, routes!(@method $method $handler)) )*
        }

        pub const ENDPOINTS: &[docs::Endpoint] = &[$(
            docs::Endpoint {
                method: docs::EndpointMethod::$method,
                path: $path,
                description: concat!( $($d, "\n",)* ),
                $(
                    $( body: Some( <$body as docs::Documentation>::DOCUMENTATION_OBJECT ), )?
                    $( response: Some( <$res as docs::Documentation>::DOCUMENTATION_OBJECT ), )?
                )?
                ..docs::_EMPTY_ENDPOINT // fills `body` and `response` with `None`
            }
        ),*
        ];
    };
}

use super::*;

routes! {
    /// Authorize and obtain token.
    POST "/auth" => auth::authorize
        :   body(auth::AuthorizationRequestData)
            res(auth::AuthorizationResponse),

    /// Get all users. Supports paging.
    /// Can be accessed by any user.
    GET "/users" => user::get_users,
    /// Get all possible roles on current instance.
    /// Can be accessed by any user.
    GET "/users/roles" => user::get_all_roles,

    GET   "/user" => user::get_self,
    PUT   "/user" => user::register,
    PATCH "/user" => user::patch_user,
    GET   "/user/spaces" => user::get_spaces,
    GET   "/user/@:user_id" => user::get_user,
    PATCH "/user/@:user_id" => user::reset_user_password,
    GET   "/user/@:user_id/role" => user::get_user_role,
    PATCH "/user/@:user_id/role" => user::promote_user,
    GET   "/user/@:user_id/spaces" => user::get_user_spaces,
    GET   "/user/invites" => user::get_invites,
    PUT   "/user/invites" => user::create_invite,
    POST  "/user/invites/wave" => user::invite_wave,

    PUT   "/space" => space::create_space,
}
