use serde::{Deserialize, Serialize};

use super::{
    macros::{impl_cuid, impl_try_from_enum},
    space::SpaceID,
};

/// Represents ID of service account (CUID)
#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Debug)]
#[serde(into = "String", try_from = "String")]
#[repr(transparent)]
pub struct ServiceAccountID(String);
impl_cuid!(ServiceAccountID);

impl_try_from_enum!(
    /// Type of service account independ of it's space
    #[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
    #[serde(into = "i64", try_from = "i64")]
    pub enum ServiceAccountTy : repr(i64) {
        /// Service that can get users by their ssh keys.
        SSHAuthority = 1,

        /// Can watch any event of space
        SpaceEventWatcher = 1000,
        /// Can report any supported type of action
        SpaceActor = 1001,
    }
);

impl ServiceAccountTy {
    /// Is space required to this type?
    pub fn is_space_required(self) -> bool {
        matches!(self, Self::SpaceEventWatcher | Self::SpaceActor)
    }

    /// Is can be created only by instance admins?
    pub fn is_admin(self) -> bool {
        matches!(self, Self::SSHAuthority)
    }
}

/// Represents service account
#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Debug)]
pub struct ServiceAccount {
    pub id: ServiceAccountID,
    pub space_id: Option<SpaceID>,
    pub ty: ServiceAccountTy,
}
