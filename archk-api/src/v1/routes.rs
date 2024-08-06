use axum::{
    routing::{delete, get},
    Router,
};

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
    GET "/users" => user::get_users
        :   res(Vec<archk::v1::user::User>),
    /// Get all possible roles on current instance.
    /// Can be accessed by any user.
    GET "/users/roles" => user::get_all_roles
        :   res(Vec<crate::roles::UserRole>),

    /// Get current user
    GET   "/user" => user::get_self
        :   res(user::SelfResponse),
    /// Register new user
    PUT   "/user" => user::register
        :   body(user::RegisterRequestData)
            res(user::RegisterResponse),
    /// Update user password
    PATCH "/user" => user::patch_user
        :   body(user::PatchUser)
            res(u64),
    /// Get own spaces. Supports paging
    GET   "/user/spaces" => user::get_spaces
        :   res(Vec<user::UserSpaceResponse>),
    /// Get other user by their ID
    GET   "/user/@:user_id" => user::get_user
        :   res(archk::v1::user::User),
    /// Reset other user password
    PATCH "/user/@:user_id" => user::reset_user_password
        :   res(user::ResetPasswordResponse),
    /// Get user role (by level)
    GET   "/user/@:user_id/role" => user::get_user_role
        :   res(crate::roles::UserRole),
    /// Promote user to role or level
    PATCH "/user/@:user_id/role" => user::promote_user
        :   body(user::PromoteUserBody)
            res(u64),
    /// Get user spaces
    GET   "/user/@:user_id/spaces" => user::get_user_spaces
        :   res(Vec<user::UserSpaceResponse>),
    /// Get invites
    GET   "/user/invites" => user::get_invites
        :   res(Vec<String>),
    /// Create invite
    PUT   "/user/invites" => user::create_invite
        :   res(String),
    /// Give every user one invite. If query param `min_level` set, gives
    /// only to users with level `min_level` or higher
    POST  "/user/invites/wave" => user::invite_wave
        :   res(u64),

    /// Create space
    PUT   "/space" => space::create_space,

    GET    "/space/:space_id" => space::get_space,
    PATCH  "/space/:space_id" => space::patch_space,
    DELETE "/space/:space_id" => space::delete_space,

    GET "/space/:space_id/account" => space::get_accounts,
    PUT "/space/:space_id/account" => space::create_account,

    GET    "/space/:space_id/account/:acc_id" => space::get_account_by_id,
    PATCH  "/space/:space_id/account/:acc_id" => space::patch_account_by_id,
    DELETE "/space/:space_id/account/:acc_id" => space::delete_account_by_id,

    GET "/space/:space_id/account/:acc_id/items" => space::get_items_of_account,

    GET "/space/:space_id/item" => space::get_items,
    PUT "/space/:space_id/item" => space::create_item,

    GET    "/space/:space_id/item/:item_id" => space::get_item_by_id,
    PATCH  "/space/:space_id/item/:item_id" => space::patch_item,
    DELETE "/space/:space_id/item/:item_id" => space::delete_item,
}
