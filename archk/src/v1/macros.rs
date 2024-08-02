macro_rules! impl_cuid {
    ($v:ident) => {
        impl $v {
            /// Generate new id
            pub fn new() -> Self {
                Self(cuid2::create_id())
            }

            /// Verify id
            pub fn from(v: String) -> Option<Self> {
                cuid2::is_cuid2(&v).then_some(Self(v))
            }
        }
        impl Default for $v {
            fn default() -> Self {
                Self::new()
            }
        }
        impl Into<String> for $v {
            fn into(self) -> String {
                self.0
            }
        }
        impl TryFrom<String> for $v {
            type Error = crate::v1::errors::StringIsNotCUID;

            fn try_from(v: String) -> Result<Self, Self::Error> {
                $v::from(v).ok_or(crate::v1::errors::StringIsNotCUID(()))
            }
        }
        impl std::ops::Deref for $v {
            type Target = str;

            fn deref(&self) -> &str {
                &self.0
            }
        }
    };
}

pub(crate) use impl_cuid;

macro_rules! impl_try_from_enum {
    ($(#[$a:meta])* $v:vis enum $name:ident : repr($i:ident) { $( $(#[$b:meta])* $variant:ident = $value:expr ),* $(,)? }) => {
        $(#[$a])*
        $v enum $name {
            $(
                $(#[$b])*
                $variant = $value,
            )*
        }

        impl TryFrom<$i> for $name {
            type Error = crate::v1::errors::NoEnumVariantError;

            fn try_from(v: $i) -> Result<Self, Self::Error> {
                match v {
                    $( $value => Ok(Self::$variant), )*
                    _ => Err(crate::v1::errors::NoEnumVariantError(())),
                }
            }
        }

        impl Into<$i> for $name {
            fn into(self) -> $i {
                self as $i
            }
        }
    };
}

pub(crate) use impl_try_from_enum;
