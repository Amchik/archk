/// Authorization models
pub mod auth;
/// Service accounts models
pub mod service;
/// Space models
pub mod space;
/// User models
pub mod user;

/// Request and response models (if different from [`archk::v1`])
pub mod models;

/// Declaration of API response structure
pub mod api;
pub mod docs;

/// Errors used in some models
pub mod errors {
    /// Invalid enum variant passed.
    ///
    /// Example: attempt to call [`TryFrom::try_from`] on value that not described in enum.
    #[derive(Debug)]
    pub struct NoEnumVariantError(pub(crate) ());

    impl std::fmt::Display for NoEnumVariantError {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "expected valid enum variant")
        }
    }

    /// Invalid CUID string.
    ///
    /// Example: attempt to call [`TryFrom::try_from`] on string that not CUID string.
    /// Used in CUID objects like [`super::user::UserID`].
    #[derive(Debug)]
    pub struct StringIsNotCUID(pub(crate) ());

    impl std::fmt::Display for StringIsNotCUID {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "expected valid CUID string")
        }
    }
}

mod macros;
