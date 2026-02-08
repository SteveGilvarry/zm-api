use fake::faker::internet::en::{Password, SafeEmail, Username};
use fake::Dummy;
use garde::Validate;
use serde::{Deserialize, Serialize};
use strum::Display;
use utoipa::{IntoParams, ToSchema};

#[derive(Debug, Deserialize, Serialize, Dummy, Validate, utoipa::ToSchema)]
pub struct RegisterRequest {
    #[dummy(faker = "Username()")]
    #[garde(ascii, length(min = 3, max = 25))]
    pub username: String,
    #[dummy(faker = "SafeEmail()")]
    #[garde(email)]
    pub email: String,
    #[dummy(faker = "Password(6..100)")]
    #[garde(length(min = 6))]
    pub password: String,
}

impl RegisterRequest {
    pub fn new(username: &str, email: &str, password: &str) -> Self {
        Self {
            password: password.to_string(),
            username: username.to_string(),
            email: email.to_string(),
        }
    }

    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(&self)
    }
}

#[derive(Debug, Deserialize, Serialize, Dummy, ToSchema, IntoParams, Clone)]
pub struct PageQueryParam {
    pub page_num: u64,
    pub page_size: u64,
    pub sort_by: Option<String>,
    pub sort_direction: Option<Direction>,
}

#[derive(
    Serialize,
    Deserialize,
    Debug,
    Display,
    Dummy,
    ToSchema,
    Clone,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
)]
pub enum Direction {
    DESC,
    ASC,
}

// TODO #![feature(unboxed_closures)] unstable
impl Direction {
    pub fn as_closure<T>(&self) -> impl Fn((T, T)) -> bool
    where
        T: Ord,
    {
        match self {
            Direction::ASC => |(a, b)| a <= b,
            Direction::DESC => |(a, b)| a >= b,
        }
    }
}

#[derive(Deserialize, Serialize, Dummy, ToSchema, Validate)]
#[serde(tag = "type")]
pub struct LoginRequest {
    #[dummy(faker = "Username()")]
    #[garde(length(min = 3, max = 64))]
    #[garde(pattern("^[a-zA-Z0-9_.-]+$"))]
    pub username: String,
    #[dummy(faker = "Password(8..64)")]
    #[garde(length(min = 6))]
    pub password: String,
}

impl std::fmt::Debug for LoginRequest {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LoginRequest")
            .field("username", &self.username)
            .field("password", &"[REDACTED]")
            .finish()
    }
}

#[derive(Serialize, Deserialize, ToSchema, Validate, Dummy, IntoParams)]
pub struct RefreshTokenRequest {
    #[garde(length(min = 30))]
    pub token: String,
}

impl std::fmt::Debug for RefreshTokenRequest {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RefreshTokenRequest")
            .field("token", &"[REDACTED]")
            .finish()
    }
}

#[derive(Serialize, Deserialize, ToSchema, Validate, Dummy, IntoParams)]
pub struct TokenInfoRequest {
    #[garde(length(min = 30))]
    pub token: String,
}

impl std::fmt::Debug for TokenInfoRequest {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TokenInfoRequest")
            .field("token", &"[REDACTED]")
            .finish()
    }
}
#[derive(Debug, Deserialize, ToSchema, Validate, Dummy, IntoParams)]
pub struct ForgetPasswordQueryParam {
    #[dummy(faker = "Username()")]
    #[garde(length(min = 3, max = 64))]
    #[garde(pattern("^[a-zA-Z0-9_.-]+$"))]
    pub username: String,
}

#[derive(Debug, Deserialize, Serialize, ToSchema, Validate, Dummy, Default)]
pub struct UpdateProfileRequest {
    #[dummy(faker = "Username()")]
    #[garde(skip)]
    pub username: Option<String>,
    #[dummy(faker = "Password(8..100)")]
    #[garde(length(min = 8))]
    pub password: Option<String>,
    #[garde(skip)]
    pub is_2fa: Option<bool>,
    #[garde(skip)]
    pub is_private: Option<bool>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_invalid_email_register_request() {
        let req = RegisterRequest::new("username", "email", "password");
        assert!(req.validate().is_err());
    }

    #[test]
    fn test_invalid_pass_register_request() {
        let req = RegisterRequest::new("username", "email@test.com", "pass");
        assert!(req.validate().is_err());
    }

    #[test]
    fn test_valid_user_register_request() {
        let req = RegisterRequest::new("foo", "foo@bar.com", "password");
        assert!(req.validate().is_ok());
    }

    #[test]
    fn test_valid_register_request() {
        let req = RegisterRequest::new("username", "email@test.com", "password");
        assert!(req.validate().is_ok());
    }
}
