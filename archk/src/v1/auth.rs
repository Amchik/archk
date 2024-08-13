//! # Authorization models
//!
//! This module contains [`auth::Token`] struct that represents a normal token.
//! Available types of tokens provided by [`auth::TokenTy`].
//!
//! ## Example usage
//!
//! ```
//! use archk::v1::auth::{Token, TokenTy};
//!
//! // Create new personal token
//! let token = Token::new(TokenTy::Personal);
//!
//! // Convert it to string
//! let token_str = token.to_string();
//!
//! // Verify token by checksum:
//! let verified_token = Token::parse(&token_str);
//! assert_eq!(Ok(token), verified_token);
//! ```

use core::fmt;
use std::time::{SystemTime, UNIX_EPOCH};

use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};

/// Type of token, used in prefixes
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TokenTy {
    /// Personal tokens, issued for users
    Personal,

    /// Tokens that belongs to service accounts.
    Service,
}

impl TokenTy {
    /// Converts [`TokenTy`] to it's prefix.
    ///
    /// # Example
    /// ```
    /// use archk::v1::auth::TokenTy;
    ///
    /// assert_eq!(TokenTy::Personal.prefix(), "acp");
    /// ```
    pub fn prefix(self) -> &'static str {
        match self {
            Self::Personal => "acp",
            Self::Service => "acs",
        }
    }
    /// Convert prefix to [`TokenTy`].
    ///
    /// # Example
    /// ```
    /// use archk::v1::auth::TokenTy;
    ///
    /// assert_eq!(TokenTy::from_prefix("acp"), Some(TokenTy::Personal));
    /// assert_eq!(TokenTy::from_prefix("unk"), None);
    /// // Prefix shouldn't contains any other symbols:
    /// assert_eq!(TokenTy::from_prefix("acp_"), None);
    /// ```
    pub fn from_prefix(prefix: &str) -> Option<Self> {
        match prefix {
            "acp" => Some(Self::Personal),
            "acs" => Some(Self::Service),
            _ => None,
        }
    }
}

/// Token raw data. Can be generated through [`Token::new`], converted to string via [`Token::to_string`]
/// and parsed by [`Token::parse`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Token {
    /// Type of token
    pub ty: TokenTy,
    /// "Issued at", timestamp in milliseconds
    pub iat: u64,
    /// Random data
    pub rnd: u32,
}

impl Token {
    /// Generate new token with given [`TokenTy`].
    pub fn new(ty: TokenTy) -> Self {
        Self {
            ty,
            iat: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("Current system time less than unix epoch")
                .as_millis() as u64,
            rnd: rand::random(),
        }
    }

    /// Parse token string to [`Token`]. Token should contain prefix
    pub fn parse(token: &str) -> Result<Self, Error> {
        let Some((prefix, token)) = token.split_once('_') else {
            return Err(Error::MissingPrefix);
        };

        let Some(ty) = TokenTy::from_prefix(prefix) else {
            return Err(Error::UnknownPrefix);
        };

        let data = match URL_SAFE_NO_PAD.decode(token) {
            Ok(data) if data.len() != 16 => return Err(Error::MalformedData),
            Ok(data) => data,
            Err(e) => return Err(Error::DecodeError(e)),
        };
        let checksum = {
            let mut buff = [0; 4];
            buff.copy_from_slice(&data[12..]);
            u32::from_le_bytes(buff)
        };

        if checksum != crc32fast::hash(&data[..12]) {
            return Err(Error::ChecksumError);
        }

        let iat = {
            let mut buff = [0; 8];
            buff.copy_from_slice(&data[..8]);
            u64::from_le_bytes(buff)
        };
        let rnd = {
            let mut buff = [0; 4];
            buff.copy_from_slice(&data[8..12]);
            u32::from_le_bytes(buff)
        };

        Ok(Self { ty, iat, rnd })
    }
}
impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut data = Vec::new();
        data.extend_from_slice(&self.iat.to_le_bytes());
        data.extend_from_slice(&self.rnd.to_le_bytes());

        let checksum = crc32fast::hash(&data);
        data.extend_from_slice(&checksum.to_le_bytes());

        let token_str = URL_SAFE_NO_PAD.encode(&data);

        write!(f, "{}_{token_str}", self.ty.prefix())
    }
}

/// Error while parsing token
#[non_exhaustive]
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Error {
    /// Missing prefix of token
    MissingPrefix,
    /// Unknown prefix of token
    UnknownPrefix,
    /// Invalid base64 decoded data
    MalformedData,
    /// Failed to decode base64
    DecodeError(base64::DecodeError),
    /// Invalid checksum
    ChecksumError,
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn generate_and_verify_token() {
        let tokens: Vec<_> = (0..16).map(|_| Token::new(TokenTy::Personal)).collect();
        let strings: Vec<_> = tokens.iter().map(|v| v.to_string()).collect();
        let verified = strings.iter().map(|v| Token::parse(v));

        for ((tok, res), s) in tokens.into_iter().zip(verified).zip(strings.iter()) {
            match res {
                Ok(res) => assert_eq!(tok, res, "String: {s}"),
                Err(e) => panic!("Failed to verify: {e:?}\nString: {s}\nToken: {tok:?}"),
            }
        }
    }
}
