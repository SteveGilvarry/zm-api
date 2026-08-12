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

// Manual Debug: the password must never reach logs, whatever the log site.
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

// Manual Debug: a refresh token is a live credential; never log it.
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

// Manual Debug: never log the token itself.
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

/// Self-service password change for the authenticated user (`PUT /me/password`).
#[derive(Deserialize, Serialize, ToSchema, Validate, Dummy)]
pub struct ChangePasswordRequest {
    #[dummy(faker = "Password(8..64)")]
    #[garde(length(min = 1))]
    pub current_password: String,
    #[dummy(faker = "Password(8..64)")]
    #[garde(length(min = 6))]
    pub new_password: String,
}

// Manual Debug: neither the current nor the new password may reach logs.
impl std::fmt::Debug for ChangePasswordRequest {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ChangePasswordRequest")
            .field("current_password", &"[REDACTED]")
            .field("new_password", &"[REDACTED]")
            .finish()
    }
}

#[derive(Deserialize, Serialize, ToSchema, Validate, Dummy, Default)]
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

// Manual Debug: the optional new password must never reach logs.
impl std::fmt::Debug for UpdateProfileRequest {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("UpdateProfileRequest")
            .field("username", &self.username)
            .field("password", &self.password.as_ref().map(|_| "[REDACTED]"))
            .field("is_2fa", &self.is_2fa)
            .field("is_private", &self.is_private)
            .finish()
    }
}

#[cfg(test)]
mod redaction_tests {
    use super::*;

    #[test]
    fn login_request_debug_redacts_password() {
        let req = LoginRequest {
            username: "admin".to_string(),
            password: "hunter2secret".to_string(),
        };
        let out = format!("{req:?}");
        assert!(out.contains("admin"));
        assert!(!out.contains("hunter2secret"));
        assert!(out.contains("[REDACTED]"));
    }

    #[test]
    fn refresh_token_request_debug_redacts_token() {
        let req = RefreshTokenRequest {
            token: "eyJhbGciOiJSUzI1NiJ9.super.secret".to_string(),
        };
        let out = format!("{req:?}");
        assert!(!out.contains("eyJhbGciOiJSUzI1NiJ9"));
        assert!(out.contains("[REDACTED]"));
    }

    #[test]
    fn token_info_request_debug_redacts_token() {
        let req = TokenInfoRequest {
            token: "eyJhbGciOiJSUzI1NiJ9.super.secret".to_string(),
        };
        let out = format!("{req:?}");
        assert!(!out.contains("eyJhbGciOiJSUzI1NiJ9"));
    }

    #[test]
    fn change_password_request_debug_redacts_both_passwords() {
        let req = ChangePasswordRequest {
            current_password: "oldsecretpw".to_string(),
            new_password: "newsecretpw".to_string(),
        };
        let out = format!("{req:?}");
        assert!(!out.contains("oldsecretpw"));
        assert!(!out.contains("newsecretpw"));
        assert!(out.contains("[REDACTED]"));
    }

    #[test]
    fn update_profile_request_debug_redacts_password() {
        let req = UpdateProfileRequest {
            username: Some("admin".to_string()),
            password: Some("hunter2secret".to_string()),
            is_2fa: None,
            is_private: None,
        };
        let out = format!("{req:?}");
        assert!(!out.contains("hunter2secret"));
        assert!(out.contains("[REDACTED]"));
    }
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
