use super::models::MayIgnored;

/// Field of struct
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DocumentationField {
    /// Field name
    pub name: &'static str,
    /// Field rust type
    pub ty: &'static str,
    /// Field description
    pub description: &'static str,
}

/// Represents [`Documentation`] but in struct. Can be obtained from [`DocumentationExt::documentation_object`].
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DocumentationObject {
    /// Type name
    pub name: &'static str,
    /// Struct fields
    pub fields: &'static [DocumentationField],

    /// Is this type array?
    pub is_array: bool,
    /// Is this type nullable?
    pub is_option: bool,
    /// Is this type may not exists in object?
    pub is_may_ignored: bool,
}

/// Described type or struct
pub trait Documentation {
    /// Type name
    const NAME: &'static str;
    /// Struct fields
    const FIELDS: &'static [DocumentationField];

    /// Is this type array?
    const IS_ARRAY: bool = false;
    /// Is this type nullable?
    const IS_OPTION: bool = false;
    /// Is this type may not exists in object?
    const IS_MAY_IGNORED: bool = false;
}

macro_rules! impl_elementary {
    ($($v:ident)*) => {
        $(
            impl Documentation for $v {
                const NAME: &'static str = stringify!($v);
                const FIELDS: &'static [DocumentationField] = &[];
            }
    )   *
    };
}
impl_elementary!(String i8 i16 i32 i64 i128 u8 u16 u32 u64 u128);

impl<T: Documentation> Documentation for Vec<T> {
    const NAME: &'static str = T::NAME;
    const FIELDS: &'static [DocumentationField] = T::FIELDS;

    const IS_ARRAY: bool = true;
    const IS_MAY_IGNORED: bool = T::IS_MAY_IGNORED;
    const IS_OPTION: bool = T::IS_OPTION;
}

impl<T: Documentation> Documentation for Option<T> {
    const NAME: &'static str = T::NAME;
    const FIELDS: &'static [DocumentationField] = T::FIELDS;

    const IS_ARRAY: bool = T::IS_ARRAY;
    const IS_OPTION: bool = true;
    const IS_MAY_IGNORED: bool = T::IS_MAY_IGNORED;
}

impl<T: Documentation> Documentation for MayIgnored<T> {
    const NAME: &'static str = T::NAME;
    const FIELDS: &'static [DocumentationField] = T::FIELDS;

    const IS_ARRAY: bool = T::IS_ARRAY;
    const IS_OPTION: bool = T::IS_OPTION;
    const IS_MAY_IGNORED: bool = true;
}

/// Obtain [`DocumentationObject`] (for example, to store in vector)
pub const fn documentation_object<T: Documentation>() -> DocumentationObject {
    DocumentationObject {
        name: T::NAME,
        fields: T::FIELDS,
        is_array: T::IS_ARRAY,
        is_option: T::IS_OPTION,
        is_may_ignored: T::IS_MAY_IGNORED,
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EndpointMethod {
    GET,
    POST,
    PUT,
    PATCH,
    DELETE,
}

pub struct Endpoint {
    pub method: EndpointMethod,
    pub path: &'static str,
    pub description: &'static str,
    pub body: Option<DocumentationObject>,
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
