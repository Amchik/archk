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

#[cfg(feature = "ssh")]
pub mod ssh {
    use serde::{Deserialize, Serialize};

    use crate::v1::{
        errors::NoEnumVariantError,
        macros::{impl_cuid, impl_try_from_enum},
    };

    /// Represents ID of user ssh key (CUID)
    #[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Debug)]
    #[repr(transparent)]
    pub struct UserSSHKeyID(String);
    impl_cuid!(UserSSHKeyID);

    impl_try_from_enum!(
        /// Supported ssh key types.
        ///
        /// # Example
        /// ```
        /// use archk::v1::user::ssh::SSHKeyTy;
        ///
        /// // Obrain from string:
        /// let ty = SSHKeyTy::try_from("ssh-rsa").ok().expect("invalid string");
        /// assert_eq!(ty, SSHKeyTy::RSA);
        ///
        /// // Convert to number:
        /// let ty_id: i64 = ty.into();
        /// assert_eq!(ty_id, 0);
        ///
        /// // Convert to string (default in (de)serializing):
        /// let ty_str: &str = SSHKeyTy::ED25519.into();
        /// assert_eq!(ty_str, "ssh-ed25519");
        /// ```
        #[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug)]
        #[repr(i64)]
        pub enum SSHKeyTy : repr(i64) {
            #[serde(rename = "ssh-rsa")]
            RSA = 0,
            #[serde(rename = "ssh-ed25519")]
            ED25519 = 1,
        }
    );

    impl<'a> TryFrom<&'a str> for SSHKeyTy {
        type Error = NoEnumVariantError;

        fn try_from(value: &'a str) -> Result<Self, NoEnumVariantError> {
            match value {
                "ssh-rsa" => Ok(Self::RSA),
                "ssh-ed25519" => Ok(Self::ED25519),
                _ => Err(NoEnumVariantError(())),
            }
        }
    }
    impl From<SSHKeyTy> for &'static str {
        fn from(value: SSHKeyTy) -> Self {
            match value {
                SSHKeyTy::ED25519 => "ssh-ed25519",
                SSHKeyTy::RSA => "ssh-rsa",
            }
        }
    }

    /// User ssh key.
    #[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Debug)]
    pub struct UserSSHKey {
        /// Key ID
        pub id: UserSSHKeyID,
        /// Key type
        pub pubkey_ty: SSHKeyTy,
        /// Key value (long public key string)
        pub pubkey_val: String,
        /// Key fingerprint hashed with sha2-256 in base64 without any prefixes
        pub pubkey_fingerprint: String,
    }

    #[derive(Debug)]
    pub enum FromPubkeyStrError {
        /// Key type not known in [`SSHKeyTy`]
        UnknownType,
        /// Invalid public key format (not in `ssh-ty <BASE64>`)
        InvalidString,
        /// Returned from [`russh_keys::parse_public_key_base64`]
        Parse(russh_keys::Error),
    }

    impl UserSSHKey {
        /// Verify and construct new [`UserSSHKey`]
        pub fn from_pubkey(pubkey_str: &str) -> Result<Self, FromPubkeyStrError> {
            let (ty, key) = {
                let mut split = pubkey_str.split(' ');

                (split.next(), split.next())
            };

            let (ty, key) = match (ty.map(SSHKeyTy::try_from), key) {
                (Some(Ok(ty)), Some(key)) => (ty, key),
                (Some(Err(_)), _) => return Err(FromPubkeyStrError::UnknownType),
                _ => todo!(),
            };

            let pubkey =
                russh_keys::parse_public_key_base64(key).map_err(FromPubkeyStrError::Parse)?;

            let fingerprint = pubkey.fingerprint();

            Ok(Self {
                id: UserSSHKeyID::new(),
                pubkey_ty: ty,
                pubkey_val: String::from(key),
                pubkey_fingerprint: fingerprint,
            })
        }
    }

    #[cfg(test)]
    mod tests {
        use super::{SSHKeyTy, UserSSHKey};

        // This test ensures that fingerprint from `ssh-keygen` or somewhere matches
        // to fingerprint from `russh_keys` (format).
        // To get fingerprint run `echo "$KEYTYPE $KEYVAL" | ssh-keygen -l -f -`:
        // ```
        // $ echo "ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAIJ..." | ssh-keygen -l -f -
        // 256 SHA256:ssIlIUznbRQkztvj/g8m7ybGlV1+1mQfbNnHo8TteJQ no comment (ED25519)
        //            ^~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~ this part
        // ```
        #[test]
        pub fn parse_ssh_public_keys() {
            let keys = [
                ("ssh-rsa AAAAB3NzaC1yc2EAAAADAQABAAABgQC1/sXPq8Dln/vYYFCjkMsNRoObcX5M2BSyRyfJsR8bujf1jaPaEMVRGLp2CpNNkaCpsz1L1ObNPgYMm4Z9aPcly5EPf4tAL12P0cTZLpZd9ohxMpBkWqs53mi4OBuvEUE7UwPPyQTZDqnJrpkSntc59p9rESVt5KC2kPGJ1vusA1QviUbHBAYu03XRBOc8FYxAQIgUYajavltJ+0+E6/YxvRtEh/eK14uZIbpJMaatcnD9VbVL6cC9RShpcOk2fen9s7mvjgH5zAVDFRx2l9xP37jYmhIevG8ByJD9fyNfKKBngImJ6yyScShguS5l2J+Y6yJqPduwUu6mkQrP37mz+CxPVEP/KHxQlrpWrK3MB6Mri37MMhAVuI51c7cxHW9R+xFHmuyljxXg/QyRwKjhNourHR7mvXQmKoIwxQuJVgWc0TOPptG2dIanzmigdDCTJE4XcX0Bb4YP21eZ1yNmU4Lnr1uuTFR/AN4iQz8TstBCbOfrXV0EaRqmch9pXh0=", SSHKeyTy::RSA, "wATXSnzsU0YBTZTY5b7EAgAuL4VlLJ/IBU2ge2tZuZE"),
                ("ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAIJBbH545J25TMANrQRrqMO/SGSc+NMHpWgvWBptO+o7A comment", SSHKeyTy::ED25519, "ssIlIUznbRQkztvj/g8m7ybGlV1+1mQfbNnHo8TteJQ")
            ];

            for (key, expected_ty, expected_fingerprint) in keys {
                let pubkey = UserSSHKey::from_pubkey(key).expect("UserSSHKey::from_pubkey");
                assert_eq!(pubkey.pubkey_ty, expected_ty);
                assert_eq!(pubkey.pubkey_fingerprint, expected_fingerprint);
            }
        }
    }
}
