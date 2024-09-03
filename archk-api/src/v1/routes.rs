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

    /// Get own SSH keys
    GET "/user/ssh-keys" => user::get_ssh_keys
        :   res(Vec<archk::v1::user::ssh::UserSSHKey>),
    /// Upload SSH key
    PUT "/user/ssh-keys" => user::upload_ssh_key
        :   body(user::UploadSSHKeyBody)
            res(archk::v1::user::ssh::UserSSHKey),
    /// Delete ssh key by their CUID
    DELETE "/user/ssh-keys/:key_id" => user::delete_ssh_key
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

    // Get logs for specific account. Supports paging.
    GET "/space/:space_id/account/:acc_id/logs" => space::get_logs_by_account
        :   res(Vec<space::SpaceItemLogEntry>),

    GET "/space/:space_id/item" => space::get_items,
    PUT "/space/:space_id/item" => space::create_item,

    GET    "/space/:space_id/item/:item_id" => space::get_item_by_id,
    PATCH  "/space/:space_id/item/:item_id" => space::patch_item,
    DELETE "/space/:space_id/item/:item_id" => space::delete_item,

    // Get logs for specific item. Supports paging.
    GET "/space/:space_id/item/:item_id/logs" => space::get_logs_by_item
        :   res(Vec<space::SpaceItemLogEntry>),

    /// Get services bound to space. Supports pagging.
    GET "/space/:space_id/services" => service::get_space_services
        :   res(Vec<service::ServiceAccountResponse>),

    /// Get admin services. If query param `?all=true` passed shows all services including from spaces.
    /// Supports paging.
    GET "/service" => service::get_services
        :   res(Vec<service::ServiceAccountResponse>),
    /// Creates new service.
    PUT "/service" => service::create_service
        // FIXME: real return type is `archk::v1::service::ServiceAccount`
        // FIXME: uncomment body() when spaces will be documentated
        :   //body(service::CreateServiceBody)
            res(service::ServiceAccountResponse),
    /// Delete service account
    DELETE "/service/:service_account_id" => service::delete_service
        :   res(u64),

    /// Get tokens count for service
    GET "/service/:service_account_id/tokens" => service::get_tokens
        :   res(i32),
    /// Issue new service token
    PUT "/service/:service_account_id/tokens" => service::put_token
        :   res(service::ServiceTokenResponse),
    /// Revoke all tokens
    DELETE "/service/:service_account_id/tokens" => service::revoke_all_tokens
        :   res(u64),

    /// Get all ssh keys matching fingerprint. Returns error no one key matches.
    POST "/service/_/ssh-keys" => service::ssh::fetch_ssh_keys_by_fingerprint
        :   body(service::ssh::FingerprintBody)
            res(Vec<service::ssh::SSHKeyResponse>),
}
