use std::borrow::Cow;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Debug)]
pub enum Response<T = NeverSerialize> {
    #[serde(rename = "response")]
    Success(T),
    #[serde(rename = "error")]
    Failture(ErrorData),
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Debug)]
pub enum NeverSerialize {}

/// Full error data, including details of error
#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Debug)]
pub struct ErrorData {
    /// Error code
    pub code: Error,
    /// Some details of error, if any
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub detail: Option<Cow<'static, str>>,
}

impl ErrorData {
    /// Appends details for error data.
    ///
    /// # Example
    /// ```
    /// use archk::v1::api::{Error, ErrorData};
    /// use std::borrow::Cow;
    ///
    /// let data: ErrorData = Error::NoEndpoint.into();
    /// let data = data.detail("Try /foo".into());
    /// assert_eq!(data.detail, Some(Cow::Borrowed("Try /foo")));
    /// ```
    pub fn detail(mut self, v: Cow<'static, str>) -> Self {
        self.detail = Some(v);
        self
    }
}

macro_rules! impl_error {
    ( $(#[$a:meta])* pub enum $e:ident { $( $(#[$b:meta])* $var:ident = $code:literal : $http:literal ),* $(,)? } ) => {
        $(#[$a])*
        pub enum $e {
            $(
                $(#[$b])*
                $var = $code,
            )*
        }

        impl $e {
            /// Returns HTTP code by error.
            pub const fn http_code(self) -> u16 {
                match self {
                    $( Self::$var => $http, )*
                }
            }
        }

        impl From<$e> for u16 {
            fn from(v: $e) -> u16 {
                v as u16
            }
        }
        impl TryFrom<u16> for $e {
            type Error = errs::InvalidValue;

            fn try_from(value: u16) -> Result<Self, Self::Error> {
                match value {
                    $( $code => Ok(Self::$var), )*
                    _ => Err(errs::InvalidValue(())),
                }
            }
        }
    };
}

impl_error!(
    /// Represents error code.
    ///
    /// # Example
    /// ```
    /// use archk::v1::api::Error;
    ///
    /// let err = Error::NoEndpoint;
    /// assert_eq!(err.http_code(), 404); // not found
    /// ```
    #[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug)]
    #[serde(into = "u16", try_from = "u16")]
    #[repr(u16)]
    pub enum Error {
        // VariantName = [it's code] : [http code],

        /// Requested object does not exists
        ObjectNotFound = 4000 : 404,
        /// Input data malformed (eg. invalid user name)
        MalformedData = 4001 : 422,
        /// Some kind of conflict (eg. non-unique username)
        Conflict = 4002 : 409,
        /// Access forbidden for resource
        Forbidden = 4003 : 403,

        /// Endpoint does not exists
        NoEndpoint = 5001 : 404,
        /// Internal error
        Internal = 5002 : 500,
        /// Invalid format of input data (eg. string to integer param)
        ProcessingError = 5003 : 415,
        /// Invalid token passed or no token passed
        Unauthorized = 5004 : 401,
    }
);

pub mod errs {
    #[derive(Copy, Clone, PartialEq, Eq, Debug)]
    pub struct InvalidValue(pub(crate) ());

    impl std::fmt::Display for InvalidValue {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.write_str("Passed value does not represents by any error variant")
        }
    }

    impl std::error::Error for InvalidValue {}
}

impl Error {
    /// Appends details for error data. Alias to [`ErrorData::detail`]
    ///
    /// # Example
    /// ```
    /// use archk::v1::api::{Error, ErrorData};
    /// use std::borrow::Cow;
    ///
    /// let data: ErrorData = Error::NoEndpoint.detail("Try /foo".into());
    /// assert_eq!(data.detail, Some(Cow::Borrowed("Try /foo")));
    /// ```
    pub fn detail(self, v: Cow<'static, str>) -> ErrorData {
        ErrorData {
            code: self,
            detail: Some(v),
        }
    }
}

impl From<Error> for ErrorData {
    fn from(code: Error) -> Self {
        Self { code, detail: None }
    }
}

#[cfg(feature = "axum")]
use axum::{http::StatusCode, response::IntoResponse, Json};

#[cfg(feature = "axum")]
impl<T: Serialize> IntoResponse for Response<T> {
    fn into_response(self) -> axum::response::Response {
        let code = match &self {
            Self::Success(_) => StatusCode::OK,
            Self::Failture(ErrorData { code, .. }) => {
                StatusCode::from_u16(code.http_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR)
            }
        };

        let mut j = Json(self).into_response();
        *j.status_mut() = code;

        j
    }
}
