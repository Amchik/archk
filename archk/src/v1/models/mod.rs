use serde::{Deserialize, Serialize};

/// Field that may be ignored on serialization/deserialization.
///
/// Default value is [`MayIgnored::Ignored`] but [`None`] in `MayIgnored<Option<T>>` will serialize into
/// [`MayIgnored::Value`] (see example).
///
/// # Example
/// ```
/// use archk::v1::models::MayIgnored;
/// use serde;
/// use serde_json;
///
/// // Fields in this struct may be number, `null` or does not exists.
/// #[derive(serde::Serialize, serde::Deserialize, PartialEq, Eq, Debug)]
/// #[allow(dead_code)]
/// struct Foo {
///     #[serde(default, skip_serializing_if="MayIgnored::is_ignored")]
///     pub foo: MayIgnored<Option<u32>>,
///     
///     #[serde(default, skip_serializing_if="MayIgnored::is_ignored")]
///     pub bar: MayIgnored<Option<u32>>,
///     
///     #[serde(default, skip_serializing_if="MayIgnored::is_ignored")]
///     pub baz: MayIgnored<Option<u32>>,
/// }
///
/// let f = Foo {
///     foo: MayIgnored::Value(Some(42)),
///     bar: MayIgnored::Value(None),
///     baz: MayIgnored::Ignored,
/// };
///
/// let j = serde_json::json!({
///     "foo": 42,
///     "bar": null,
///     // no "baz" field
/// });
///
/// let f2: Foo = serde_json::from_value(j).unwrap();
/// assert_eq!(f2, f);
/// ```
/// JSON of `f` will be:
/// ```json
/// {
///     "foo": 42,
///     "bar": null
/// }
/// ```
#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Debug)]
#[serde(untagged)]
pub enum MayIgnored<T> {
    Value(T),
    Ignored,
}

impl<T> MayIgnored<T> {
    /// Returns `true` if field ignored.
    ///
    /// # Example
    /// ```
    /// use archk::v1::models::MayIgnored;
    ///
    /// let v = MayIgnored::<u32>::Ignored;
    /// assert!(v.is_ignored());
    /// ```
    pub fn is_ignored(&self) -> bool {
        matches!(self, Self::Ignored)
    }

    /// Returns `true` if field not ignored.
    ///
    /// # Example
    /// ```
    /// use archk::v1::models::MayIgnored;
    ///
    /// let v = MayIgnored::Value(42);
    /// assert!(v.is_value());
    /// ```
    pub fn is_value(&self) -> bool {
        matches!(self, Self::Value(_))
    }

    /// Converts [`MayIgnored`] to [`Option`]
    pub fn ok(self) -> Option<T> {
        match self {
            Self::Value(v) => Some(v),
            Self::Ignored => None,
        }
    }
}

impl<T> Default for MayIgnored<T> {
    fn default() -> Self {
        Self::Ignored
    }
}
