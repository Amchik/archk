use std::time::{SystemTime, UNIX_EPOCH};

use documentation_macro::Documentation;
use once_cell::sync::Lazy;
use regex::Regex;
use serde::{Deserialize, Serialize};

use super::{docs::impl_documentation, macros::impl_cuid};

/// Represents ID of user (CUID)
#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Debug)]
#[repr(transparent)]
pub struct UserID(String);
impl_cuid!(UserID);
impl_documentation!(UserID);

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Debug, Documentation)]
pub struct User {
    /// CUID of user
    pub id: UserID,

    /// User name
    pub name: String,

    /// Who invited user? If any
    pub invited_by: Option<String>,
}
/// Represents ID of telegram user authorization request (CUID)
#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Debug)]
#[repr(transparent)]
pub struct UserTelegramAuthID(String);
impl_cuid!(UserTelegramAuthID);

/// Represent a authorization request throught telegram
pub struct UserTelegramAuth {
    pub id: UserTelegramAuthID,
    pub user_id: UserID,
    pub issued_at: u64,
}

impl UserTelegramAuth {
    /// Max wait time of request
    const WAIT_TIME_MS: u64 = 1000 * 60 * 10; // 10 min

    /// Generate new code
    pub fn new(user_id: UserID) -> Self {
        Self {
            id: UserTelegramAuthID::new(),
            user_id,
            issued_at: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("Current system time less than UNIX epoch")
                .as_millis() as u64,
        }
    }

    /// Is code actual?
    pub fn is_actual(&self) -> bool {
        let current = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Current system time less than UNIX epoch")
            .as_millis() as u64;
        self.issued_at + UserTelegramAuth::WAIT_TIME_MS >= current
    }
}

/// Check is username valid
///
/// # Examples
/// ```
/// use archk::v1::user::is_valid_username;
///
/// assert!(is_valid_username("greg")); // a valid username
/// assert!(is_valid_username("greg.b42")); // also valid username
///
/// assert!(!is_valid_username("gr")); // too small
/// assert!(!is_valid_username("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa")); // too long (>31 symbols)
/// assert!(!is_valid_username("he-llo world")); // incorrect chars
/// ```
pub fn is_valid_username(v: &str) -> bool {
    static RE: Lazy<Regex> =
        Lazy::new(|| Regex::new(r"^[a-zA-Z0-9\.]{3,31}$").expect("regex user::is_valid_username"));

    RE.is_match(v)
}
