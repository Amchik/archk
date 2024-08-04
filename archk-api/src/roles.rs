use archk::Documentation;
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct UserRoles(pub Vec<UserRole>);

impl UserRoles {
    /// Get max role by level (admin role)
    pub fn get_max(&self) -> &UserRole {
        let mut max: Option<&UserRole> = None;
        for role in self.0.iter() {
            if max.filter(|v| v.level > role.level).is_none() {
                max = Some(role);
            }
        }
        max.expect("No user roles")
    }

    /// Get maximum role by current level
    pub fn get_current(&self, level: i64) -> Option<&UserRole> {
        let mut max: Option<&UserRole> = None;
        for role in self.0.iter().filter(|v| v.level <= level) {
            if max.filter(|v| v.level > role.level).is_none() {
                max = Some(role);
            }
        }
        max
    }
}

#[derive(Serialize, Deserialize, Documentation)]
pub struct UserRole {
    pub name: String,
    pub level: i64,
    #[serde(default)]
    pub permissions: RolePermissions,
}

#[derive(Serialize, Deserialize, Default, Documentation)]
pub struct RolePermissions {
    /// Promote users to current role or demote if role less than current.
    #[serde(default)]
    pub promote: bool,
    /// Access to make new invite waves (give invites to many/all users)
    #[serde(default)]
    pub wave: bool,
    /// Access to reset users passwords and drop users
    #[serde(default)]
    pub manage: bool,

    /// Can create spaces?
    #[serde(default)]
    pub spaces: bool,
    /// Can manage spaces?
    #[serde(default)]
    pub spaces_manage: bool,
}
