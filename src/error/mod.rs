use axum::{
  http::StatusCode,
  response::{IntoResponse, Response},
  Json,
};
use serde::Deserialize;
use serde::Serialize;
use strum::EnumString;
use utoipa::ToSchema;

use crate::entity;

pub type AppResult<T = ()> = std::result::Result<T, AppError>;

#[derive(Debug, thiserror::Error, ToSchema)]
pub enum AppError {
  #[error("{0} not found")]
  NotFoundError(Resource),
  #[error("{0} not available")]
  NotAvailableError(Resource),
  #[error("{0} already exists")]
  ResourceExistsError(Resource),
  #[error("{0}")]
  PermissionDeniedError(String),
  #[error("{0}")]
  UserNotActiveError(String),
  #[error("{0}")]
  InvalidSessionError(String),
  #[error("{0}")]
  ConflictError(String),
  #[error("{0}")]
  UnauthorizedError(String),
  #[error("bad request {0}")]
  BadRequestError(String),
  #[error("{0}")]
  InvalidPayloadError(String),
  #[error("{0}")]
  HashError(String),
  #[error("internal server error: {0}")]
  InternalServerError(String),
  #[error("service unavailable: {0}")]
  ServiceUnavailableError(String),
  
  #[error(transparent)]
  #[schema(value_type = String, example = "Validation failed")]
  InvalidInputError(#[from] garde::Report),
  
  #[error(transparent)]
  #[schema(value_type = String, example = "Database error")]
  DatabaseError(#[from] sea_orm::error::DbErr),
  
  #[error(transparent)]
  #[schema(value_type = String, example = "WebSocket error")]
  WebSocketError(#[from] tokio_tungstenite::tungstenite::Error),
  
  #[error(transparent)]
  #[schema(value_type = String, example = "IO error")]
  IoError(#[from] std::io::Error),
  
  #[error(transparent)]
  #[schema(value_type = String, example = "UUID error")]
  UuidError(#[from] uuid::Error),
  
  #[error(transparent)]
  #[schema(value_type = String, example = "JWT error")]
  JwtError(#[from] jsonwebtoken::errors::Error),
  
  #[error(transparent)]
  #[schema(value_type = String, example = "HTTP client error")]
  HttpClientError(#[from] reqwest::Error),
  
  
  #[error(transparent)]
  #[schema(value_type = String, example = "Config error")]
  ConfigError(#[from] config::ConfigError),
  
  #[error(transparent)]
  #[schema(value_type = String, example = "SMTP error")]
  SmtpError(#[from] lettre::transport::smtp::Error),
  
  #[error(transparent)]
  #[schema(value_type = String, example = "Letter error")]
  LetterError(#[from] lettre::error::Error),
  
  #[error(transparent)]
  #[schema(value_type = String, example = "JSON parse error")]
  ParseJsonError(#[from] serde_json::Error),
  
  #[error(transparent)]
  #[schema(value_type = String, example = "Float parse error")]
  ParseFloatError(#[from] std::num::ParseFloatError),
  
  #[error(transparent)]
  #[schema(value_type = String, example = "Address parse error")]
  AddrParseError(#[from] std::net::AddrParseError),
  
  #[error(transparent)]
  #[schema(value_type = String, example = "Task join error")]
  SpawnTaskError(#[from] tokio::task::JoinError),
  
  #[error(transparent)]
  #[schema(value_type = String, example = "Template error")]
  TeraError(#[from] tera::Error),
  
  #[error(transparent)]
  #[schema(value_type = String, example = "Base64 decode error")]
  Base64Error(#[from] base64::DecodeError),
  
  #[error(transparent)]
  #[schema(value_type = String, example = "Enum parse error")]
  StrumParseError(#[from] strum::ParseError),
  
  #[error(transparent)]
  #[schema(value_type = String, example = "System time error")]
  SystemTimeError(#[from] std::time::SystemTimeError),
  
  #[error(transparent)]
  #[schema(value_type = String, example = "Axum error")]
  AxumError(#[from] axum::Error),
  
  #[error(transparent)]
  #[schema(value_type = String, example = "Unknown error")]
  UnknownError(#[from] anyhow::Error),
  
  #[error(transparent)]
  #[schema(value_type = String, example = "Infallible error")]
  Infallible(#[from] std::convert::Infallible),
  
  #[error(transparent)]
  #[schema(value_type = String, example = "Typed header error")]
  TypeHeaderError(#[from] axum_extra::typed_header::TypedHeaderRejection),
}

impl From<argon2::password_hash::Error> for AppError {
  fn from(value: argon2::password_hash::Error) -> Self {
    AppError::HashError(value.to_string())
  }
}

impl From<bcrypt::BcryptError> for AppError {
  fn from(value: bcrypt::BcryptError) -> Self {
    AppError::HashError(value.to_string())
  }
}

impl AppError {
  pub fn response(self) -> (StatusCode, AppResponseError) {
    use AppError::*;
    let message = self.to_string();
    let (kind, code, details, status_code) = match self {
      InvalidPayloadError(_err) => (
        "INVALID_PAYLOAD_ERROR".to_string(),
        None,
        vec![],
        StatusCode::BAD_REQUEST,
      ),
      BadRequestError(_err) => (
        "BAD_REQUEST_ERROR".to_string(),
        None,
        vec![],
        StatusCode::BAD_REQUEST,
      ),
      InternalServerError(_err) => (
        "INTERNAL_SERVER_ERROR".to_string(),
        None,
        vec![],
        StatusCode::INTERNAL_SERVER_ERROR,
      ),
      ServiceUnavailableError(_err) => (
        "SERVICE_UNAVAILABLE_ERROR".to_string(),
        None,
        vec![],
        StatusCode::SERVICE_UNAVAILABLE,
      ),
      NotAvailableError(resource) => (
        format!("{resource}_NOT_AVAILABLE_ERROR"),
        None,
        vec![],
        StatusCode::NOT_FOUND,
      ),
      NotFoundError(resource) => (
        format!("{resource}_NOT_FOUND_ERROR"),
        Some(resource.resource_type as i32),
        resource.details.clone(),
        StatusCode::NOT_FOUND,
      ),
      ResourceExistsError(resource) => (
        format!("{resource}_ALREADY_EXISTS_ERROR"),
        Some(resource.resource_type as i32),
        resource.details.clone(),
        StatusCode::CONFLICT,
      ),
      AxumError(_err) => (
        "AXUM_ERROR".to_string(),
        None,
        vec![],
        StatusCode::INTERNAL_SERVER_ERROR,
      ),
      ConfigError(_err) => (
        "CONFIG_ERROR".to_string(),
        None,
        vec![],
        StatusCode::INTERNAL_SERVER_ERROR,
      ),
      AddrParseError(_err) => (
        "ADDR_PARSE_ERROR".to_string(),
        None,
        vec![],
        StatusCode::INTERNAL_SERVER_ERROR,
      ),
      IoError(err) => {
        let (status, kind, code) = match err.kind() {
          std::io::ErrorKind::NotFound => (
            StatusCode::NOT_FOUND,
            format!("{}_NOT_FOUND_ERROR", ResourceType::File),
            Some(ResourceType::File as i32),
          ),
          std::io::ErrorKind::PermissionDenied => {
            (StatusCode::FORBIDDEN, "FORBIDDEN_ERROR".to_string(), None)
          }
          _ => (
            StatusCode::INTERNAL_SERVER_ERROR,
            "IO_ERROR".to_string(),
            None,
          ),
        };
        (kind, code, vec![], status)
      }
      WebSocketError(_err) => (
        "WEBSOCKET_ERROR".to_string(),
        None,
        vec![],
        StatusCode::INTERNAL_SERVER_ERROR,
      ),
      ParseJsonError(_err) => (
        "PARSE_JSON_ERROR".to_string(),
        None,
        vec![],
        StatusCode::INTERNAL_SERVER_ERROR,
      ),
      StrumParseError(_err) => (
        "STRUM_PARSE_ERROR".to_string(),
        None,
        vec![],
        StatusCode::INTERNAL_SERVER_ERROR,
      ),
      HttpClientError(_err) => (
        "HTTP_CLIENT_ERROR".to_string(),
        None,
        vec![],
        StatusCode::INTERNAL_SERVER_ERROR,
      ),
      SystemTimeError(_err) => (
        "SYSTEM_TIME_ERROR".to_string(),
        None,
        vec![],
        StatusCode::INTERNAL_SERVER_ERROR,
      ),
      SpawnTaskError(_err) => (
        "SPAWN_TASK_ERROR".to_string(),
        None,
        vec![],
        StatusCode::INTERNAL_SERVER_ERROR,
      ),
      UnknownError(_err) => (
        "UNKNOWN_ERROR".to_string(),
        None,
        vec![],
        StatusCode::INTERNAL_SERVER_ERROR,
      ),
      PermissionDeniedError(_err) => (
        "PERMISSION_DENIED_ERROR".to_string(),
        None,
        vec![],
        StatusCode::FORBIDDEN,
      ),
      InvalidSessionError(_err) => (
        "INVALID_SESSION_ERROR".to_string(),
        None,
        vec![],
        StatusCode::BAD_REQUEST,
      ),
      ConflictError(_err) => (
        "CONFLICT_ERROR".to_string(),
        None,
        vec![],
        StatusCode::INTERNAL_SERVER_ERROR,
      ),
      UserNotActiveError(_err) => (
        "USER_NOT_ACTIVE_ERROR".to_string(),
        None,
        vec![],
        StatusCode::FORBIDDEN,
      ),
      UnauthorizedError(_err) => (
        "UNAUTHORIZED_ERROR".to_string(),
        None,
        vec![],
        StatusCode::UNAUTHORIZED,
      ),
      UuidError(_err) => (
        "UUID_ERROR".to_string(),
        None,
        vec![],
        StatusCode::INTERNAL_SERVER_ERROR,
      ),
      JwtError(_err) => (
        "UNAUTHORIZED_ERROR".to_string(),
        None,
        vec![],
        StatusCode::UNAUTHORIZED,
      ),
      SmtpError(_err) => (
        "SMTP_ERROR".to_string(),
        None,
        vec![],
        StatusCode::INTERNAL_SERVER_ERROR,
      ),
      LetterError(_err) => (
        "LETTER_ERROR".to_string(),
        None,
        vec![],
        StatusCode::INTERNAL_SERVER_ERROR,
      ),
      HashError(_err) => (
        "HASH_ERROR".to_string(),
        None,
        vec![],
        StatusCode::INTERNAL_SERVER_ERROR,
      ),
      ParseFloatError(_err) => (
        "PARSE_FLOAT_ERROR".to_string(),
        None,
        vec![],
        StatusCode::INTERNAL_SERVER_ERROR,
      ),
      TeraError(_err) => (
        "TERA_ERROR".to_string(),
        None,
        vec![],
        StatusCode::INTERNAL_SERVER_ERROR,
      ),
      Base64Error(_err) => (
        "BASE64_ERROR".to_string(),
        None,
        vec![],
        StatusCode::INTERNAL_SERVER_ERROR,
      ),
      InvalidInputError(err) => (
        "INVALID_INPUT_ERROR".to_string(),
        None,
        err
          .iter()
          .map(|(p, e)| (p.to_string(), e.to_string()))
          .collect(),
        StatusCode::BAD_REQUEST,
      ),
      DatabaseError(_err) => (
        "DATABASE_ERROR".to_string(),
        None,
        vec![],
        StatusCode::INTERNAL_SERVER_ERROR,
      ),
      Infallible(_err) => (
        "INFALLIBLE".to_string(),
        None,
        vec![],
        StatusCode::INTERNAL_SERVER_ERROR,
      ),
      TypeHeaderError(_err) => (
        "TYPE_HEADER_ERROR".to_string(),
        None,
        vec![],
        StatusCode::INTERNAL_SERVER_ERROR,
      ),
    };

    (
      status_code,
      AppResponseError::new(kind, message, code, details),
    )
  }
}

impl IntoResponse for AppError {
  fn into_response(self) -> Response {
    let (status_code, body) = self.response();
    (status_code, Json(body)).into_response()
  }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, utoipa::ToSchema)]
pub struct AppResponseError {
  pub kind: String,
  pub error_message: String,
  pub code: Option<i32>,
  pub details: Vec<(String, String)>,
}

impl AppResponseError {
  pub fn new(
    kind: impl Into<String>,
    message: impl Into<String>,
    code: Option<i32>,
    details: Vec<(String, String)>,
  ) -> Self {
    Self {
      kind: kind.into(),
      error_message: message.into(),
      code,
      details,
    }
  }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, ToSchema)]
pub struct Resource {
  pub details: Vec<(String, String)>,
  pub resource_type: ResourceType,
}

impl std::fmt::Display for Resource {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    // TODO
    self.resource_type.fmt(f)
  }
}

#[derive(Debug, EnumString, strum::Display, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ToSchema)]
pub enum ResourceType {
  #[strum(serialize = "USER")]
  User,
  #[strum(serialize = "FILE")]
  File,
  #[strum(serialize = "SESSION")]
  Session,
  #[strum(serialize = "MESSAGE")]
  Message,
  #[strum(serialize = "MONITOR")]
  Monitor,
  #[strum(serialize = "CONFIG")]
  Config,
  #[strum(serialize = "EVENT_TAG")]
  EventTag,
}

pub fn invalid_input_error(field: &'static str, message: &'static str) -> AppError {
  let mut report = garde::Report::new();
  report.append(garde::Path::new(field), garde::Error::new(message));
  AppError::InvalidInputError(report)
}

#[allow(clippy::result_large_err)]
pub trait ToAppResult {
  type Output: entity::AppEntity;
  fn to_result(self) -> AppResult<Self::Output>;
  fn check_absent(self) -> AppResult;
  fn check_absent_details(self, details: Vec<(String, String)>) -> AppResult;
  fn to_result_details(self, details: Vec<(String, String)>) -> AppResult<Self::Output>;
}

impl<T> ToAppResult for Option<T>
where
  T: entity::AppEntity,
{
  type Output = T;
  fn to_result(self) -> AppResult<Self::Output> {
    self.ok_or_else(|| {
      AppError::NotFoundError(Resource {
        details: vec![],
        resource_type: Self::Output::RESOURCE,
      })
    })
  }

  fn to_result_details(self, details: Vec<(String, String)>) -> AppResult<Self::Output> {
    self.ok_or_else(|| {
      AppError::NotFoundError(Resource {
        details,
        resource_type: Self::Output::RESOURCE,
      })
    })
  }

  fn check_absent(self) -> AppResult {
    if self.is_some() {
      Err(AppError::ResourceExistsError(Resource {
        details: vec![],
        resource_type: Self::Output::RESOURCE,
      }))
    } else {
      Ok(())
    }
  }

  fn check_absent_details(self, details: Vec<(String, String)>) -> AppResult {
    if self.is_some() {
      Err(AppError::ResourceExistsError(Resource {
        details,
        resource_type: Self::Output::RESOURCE,
      }))
    } else {
      Ok(())
    }
  }
}
