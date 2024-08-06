//! # Endpoint Documentation
//!
//! This module contains some basic traits and structs to describe API `v1` endpoints.
//!
//! - [`Documentation`] trait: any type that implements in can be described in documentation
//! - [`DocumentationObject`] struct: describes object that implements [`Documentation`] trait
//! - [`Endpoint`] struct: describes API endpoint.
//!
//! ## Documentation trait
//!
//! [`Documentation`] trait can be implemented using [`archk::Documentation`] derive macro (required `derive` feature).
//!
//! [`Documentation`] already implemented on some basic types, like `String`, integers, floats, and ids in this crate.
//! Also there some implementation for containers:
//!
//! - [`Vec<T>`] if `T: Documentation`: array
//! - [`Option<T>`] if `T: Documentation`: nullable
//! - [`MayIgnored<T>`] if `T: Documentation`: if may ignored in serde
//!
//! ## Examples
//!
//! ### Documentate basic object
//!
//! ```ignore
//! pub struct MyID(String);
//!
//! docs::impl_documentation!(MyID); // impls Documentation as some elementary type like `u32`
//! assert_eq!(<MyID as Documentation>::DOCUMENTATION_OBJECT.name, "MyID");
//! ```
//!
//! ### Documentate structure
//! Use derive macro [`archk::Documentation`].
//! ```ignore
//! use archk::v1::{docs::{Documentation, DocumentationObject}, models::MayIgnored};
//!
//! #[derive(archk::Documentation)]
//! pub struct SomeBody {
//!     /// Some description for `usernames`
//!     pub usernames: Vec<String>,
//!     /// ...
//!     pub number_or_null: Option<u32>,
//!     /// yeeeeeeee
//!     pub may_not_exists_or_null: MayIgnored<Option<String>>,
//! }
//!
//! let object = <SomeBody as Documentation>::DOCUMENTATION_OBJECT;
//! assert_eq!(object.fields.len(), 3);
//! ```
//!

use serde::Serialize;

use super::models::MayIgnored;

/// Field of struct
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct DocumentationField {
    /// Field name
    pub name: &'static str,
    /// Field documentation
    pub documentation: DocumentationObject,
}

/// Represents [`Documentation`] object. Contains basic information about type.
///
/// # Example
/// ```
/// use archk::v1::docs::Documentation;
///
/// let object = <Vec<String> as Documentation>::DOCUMENTATION_OBJECT;
/// assert_eq!(object.name, "String");
/// assert!(object.is_array);
/// ```
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct DocumentationObject {
    /// Base type name.
    ///
    /// **Note**: it doesn't contains any prefixes or suffixes like `?` or `[]`. See [`DocumentationObject`] example.
    pub name: &'static str,
    /// Field (or base type) description
    pub description: &'static str,
    /// Struct fields
    pub fields: &'static [DocumentationField],

    /// Is this type array? Usually covered into [`Vec`]
    pub is_array: bool,
    /// Is this type nullable? Usually covered into [`Option`]
    pub is_option: bool,
    /// Is this type may not exists in object?
    /// See [`MayIgnored`] for more.
    pub is_may_ignored: bool,
}

impl DocumentationObject {
    /// Creates new instance of [`DocumentationObject`]. See struct description
    /// for more.
    pub const fn new(
        name: &'static str,
        description: &'static str,
        fields: &'static [DocumentationField],
    ) -> Self {
        Self {
            name,
            description,
            fields,
            is_array: false,
            is_option: false,
            is_may_ignored: false,
        }
    }

    /// Constructor set. See [`DocumentationObject`] documentation for more.
    pub const fn set_array(mut self, is_array: bool) -> Self {
        self.is_array = is_array;
        self
    }
    /// Constructor set. See [`DocumentationObject`] documentation for more.
    pub const fn set_option(mut self, is_option: bool) -> Self {
        self.is_option = is_option;
        self
    }
    /// Constructor set. See [`DocumentationObject`] documentation for more.
    pub const fn set_may_ignored(mut self, is_may_ignored: bool) -> Self {
        self.is_may_ignored = is_may_ignored;
        self
    }
    /// Constructor set. See [`DocumentationObject`] documentation for more.
    pub const fn set_description(mut self, description: &'static str) -> Self {
        self.description = description;
        self
    }
}

/// Described type or struct.
///
/// See module level documentation or [`DocumentationObject`] for more.
pub trait Documentation {
    /// Documentation object
    const DOCUMENTATION_OBJECT: DocumentationObject;
}

/// Generate very basic [`Documentation`] trait implementation.
///
/// # Usage
/// ```ignore
/// struct MyType;
/// impl_documentation!(MyType);
///
/// assert_eq!(<MyType as Documentation>::DOCUMENTATION_OBJECT.name, "MyType");
/// ```
macro_rules! impl_documentation {
    ($($v:ident)*) => {
        $(
            impl crate::v1::docs::Documentation for $v {
                const DOCUMENTATION_OBJECT: crate::v1::docs::DocumentationObject = crate::v1::docs::DocumentationObject::new(stringify!($v), "", &[]);
            }
    )   *
    };
}
pub(crate) use impl_documentation;
impl_documentation!(String i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 bool);

impl<T: Documentation> Documentation for Vec<T> {
    const DOCUMENTATION_OBJECT: DocumentationObject = T::DOCUMENTATION_OBJECT.set_array(true);
}

impl<T: Documentation> Documentation for Option<T> {
    const DOCUMENTATION_OBJECT: DocumentationObject = T::DOCUMENTATION_OBJECT.set_option(true);
}

impl<T: Documentation> Documentation for MayIgnored<T> {
    const DOCUMENTATION_OBJECT: DocumentationObject = T::DOCUMENTATION_OBJECT.set_may_ignored(true);
}

/// Represents endpoint method used in autogenerated documentation.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub enum EndpointMethod {
    GET,
    POST,
    PUT,
    PATCH,
    DELETE,
}
impl std::fmt::Display for EndpointMethod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self {
            Self::GET => write!(f, "GET"),
            Self::POST => write!(f, "POST"),
            Self::PATCH => write!(f, "PATCH"),
            Self::PUT => write!(f, "PUT"),
            Self::DELETE => write!(f, "DELETE"),
        }
    }
}

/// Describes API endpoint in `v1`.
#[derive(Clone, Debug, Serialize)]
pub struct Endpoint {
    /// HTTP method
    pub method: EndpointMethod,
    /// Relative path. (ex. `/user/:user_id` stands for `/api/v1/user/<user id>`)
    pub path: &'static str,
    /// Endpoint description. Supports markdown
    pub description: &'static str,
    /// Body documentation if required
    pub body: Option<DocumentationObject>,
    /// Response documentation if available
    pub response: Option<DocumentationObject>,
}

// Pseudo-Default implementation of Endpoint. `method`, `path` and `description` should be filled.
// Used only in macroses. Subject to remove
#[doc(hidden)]
pub const _EMPTY_ENDPOINT: Endpoint = Endpoint {
    method: EndpointMethod::GET,
    path: "",
    description: "",
    body: None,
    response: None,
};
