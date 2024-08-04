use super::models::MayIgnored;

/// Field of struct
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DocumentationField {
    /// Field name
    pub name: &'static str,
    /// Field documentation
    pub documentation: DocumentationObject,
}

/// Represents [`Documentation`] object.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DocumentationObject {
    /// Type name
    pub name: &'static str,
    /// Type (field) description
    pub description: &'static str,
    /// Struct fields
    pub fields: &'static [DocumentationField],

    /// Is this type array?
    pub is_array: bool,
    /// Is this type nullable?
    pub is_option: bool,
    /// Is this type may not exists in object?
    pub is_may_ignored: bool,
}

impl DocumentationObject {
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

    pub const fn set_array(mut self, is_array: bool) -> Self {
        self.is_array = is_array;
        self
    }
    pub const fn set_option(mut self, is_option: bool) -> Self {
        self.is_option = is_option;
        self
    }
    pub const fn set_may_ignored(mut self, is_may_ignored: bool) -> Self {
        self.is_may_ignored = is_may_ignored;
        self
    }
    pub const fn set_description(mut self, description: &'static str) -> Self {
        self.description = description;
        self
    }
}

/// Described type or struct
pub trait Documentation {
    /// Documentation object
    const DOCUMENTATION_OBJECT: DocumentationObject;
}

macro_rules! impl_elementary {
    ($($v:ident)*) => {
        $(
            impl Documentation for $v {
                const DOCUMENTATION_OBJECT: DocumentationObject = DocumentationObject::new(stringify!($v), "", &[]);
            }
    )   *
    };
}
impl_elementary!(String i8 i16 i32 i64 i128 u8 u16 u32 u64 u128);

impl<T: Documentation> Documentation for Vec<T> {
    const DOCUMENTATION_OBJECT: DocumentationObject = T::DOCUMENTATION_OBJECT.set_array(true);
}

impl<T: Documentation> Documentation for Option<T> {
    const DOCUMENTATION_OBJECT: DocumentationObject = T::DOCUMENTATION_OBJECT.set_option(true);
}

impl<T: Documentation> Documentation for MayIgnored<T> {
    const DOCUMENTATION_OBJECT: DocumentationObject = T::DOCUMENTATION_OBJECT.set_may_ignored(true);
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
