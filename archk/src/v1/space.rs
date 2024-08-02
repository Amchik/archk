use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::{
    macros::{impl_cuid, impl_try_from_enum},
    user::UserID,
};

/// Represents ID of space (CUID)
#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Debug)]
#[serde(into = "String", try_from = "String")]
#[repr(transparent)]
pub struct SpaceID(String);
impl_cuid!(SpaceID);

/// Represents ID of item in space (CUID)
#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Debug)]
#[repr(transparent)]
pub struct SpaceItemID(String);
impl_cuid!(SpaceItemID);

/// Represents space object
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Space {
    pub id: SpaceID,
    pub title: String,
    pub owner_id: UserID,
}

/// Represents account in space
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SpaceAccount {
    /// Account unique ID given by platform.
    /// ID unique only in current space.
    pub pl_id: String,
    /// Space ID
    pub space_id: SpaceID,

    /// Formal name given by platform
    pub pl_name: Option<String>,
    /// Display name given by platform
    pub pl_displayname: Option<String>,
}

impl_try_from_enum!(
    /// Type of item in space
    #[derive(Serialize, Deserialize, Clone, Copy, Debug, Default, PartialEq, Eq)]
    #[serde(into = "i64", try_from = "i64")]
    pub enum SpaceItemTy : repr(i64) {
        /// Normal item
        #[default]
        Normal = 0,

        /// Keycard
        Keycard = 1,
    }
);

impl SpaceItemTy {
    /// Is this item type always belongs to some user?
    ///
    /// # Example
    /// ```
    /// use archk::v1::space::SpaceItemTy;
    ///
    /// // Keycard belongs to user
    /// assert!(SpaceItemTy::Keycard.is_owner_required());
    /// // Normal (general) item may or may not belongs to user
    /// assert!(!SpaceItemTy::Normal.is_owner_required());
    /// ```
    pub fn is_owner_required(self) -> bool {
        match self {
            Self::Normal => false,
            Self::Keycard => true,
        }
    }
}
impl std::fmt::Display for SpaceItemTy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Normal => write!(f, "normal"),
            Self::Keycard => write!(f, "keycard"),
        }
    }
}

/// Represents item in space
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SpaceItem {
    /// Global item ID in all spaces
    pub id: SpaceItemID,
    /// Item title
    pub title: String,
    /// Item type
    pub ty: SpaceItemTy,

    /// Serial ID of item given by platform
    pub pl_serial: String,

    /// Platform ID of owner (see `pl_id` in [`SpaceAccount`])
    pub owner_id: Option<String>,
    /// Space ID of item and it's owner
    pub space_id: SpaceID,
}

impl_try_from_enum!(
    /// Action from space logs
    #[derive(Serialize, Deserialize, Clone, Copy, Debug)]
    #[serde(into = "i64", try_from = "i64")]
    pub enum SpaceLogAction : repr(i64) {
        KeycardScanned = 100,
        ItemTaken = 200,
        ItemReturned = 300,
    }
);

/// Space log entry.
///
/// # Example
/// ```
/// use archk::v1::space::{SpaceID, SpaceItemID, SpaceLog, SpaceLogAction};
///
/// let space_id = SpaceID::new();
/// let log = SpaceLog::new(space_id, SpaceLogAction::KeycardScanned)
///     .with_account("platform-example-account-id".to_string());
/// assert_eq!(log.sp_item_id, None);
///
/// let log = log.with_item(SpaceItemID::new());
/// assert!(matches!(log.sp_item_id, Some(_)));
/// ```
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SpaceLog {
    /// Global space log ID (usually represent as UUIDv4)
    pub id: String,
    /// Space ID of this entry
    pub space_id: SpaceID,
    /// Creation timestamp
    pub created_at: i64,

    /// Action
    pub act: SpaceLogAction,
    /// Account platform ID (see `pl_id` in [`SpaceAccount`]) if any
    pub sp_acc_id: Option<String>,
    /// Item ID if any
    pub sp_item_id: Option<SpaceItemID>,
}

impl SpaceLog {
    /// Creates empty log record. See [`SpaceLog`] docs for more
    pub fn new(space_id: SpaceID, act: SpaceLogAction) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            space_id,
            created_at: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("Time since UNIX EPOCH")
                .as_millis() as i64,
            act,
            sp_acc_id: None,
            sp_item_id: None,
        }
    }

    /// Assigns `sp_acc_id`. See [`SpaceLog`] docs for more
    pub fn with_account(mut self, sp_acc_id: String) -> Self {
        self.sp_acc_id = Some(sp_acc_id);
        self
    }

    /// Assigns `sp_item_id`. See [`SpaceLog`] docs for more
    pub fn with_item(mut self, sp_item_id: SpaceItemID) -> Self {
        self.sp_item_id = Some(sp_item_id);
        self
    }
}
