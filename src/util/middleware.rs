use axum::{extract::Request, http::StatusCode, middleware::Next, response::Response};
use tracing::{debug, info, warn};

use crate::{constant::ACCESS_TOKEN_DECODE_KEY, util::claim::UserClaims};

/// Middleware to verify JWT token for protected routes
pub async fn auth_middleware(
    request: Request,
    next: Next,
) -> Result<Response, (StatusCode, String)> {
    info!("Authenticating request: {}", request.uri());

    // Extract the authorization header
    let auth_header = request
        .headers()
        .get("Authorization")
        .and_then(|header| header.to_str().ok())
        .ok_or_else(|| {
            warn!("Missing Authorization header");
            (
                StatusCode::UNAUTHORIZED,
                "Missing Authorization header".to_string(),
            )
        })?;

    // Check if it's a Bearer token
    if !auth_header.starts_with("Bearer ") {
        warn!("Invalid Authorization header format");
        return Err((
            StatusCode::UNAUTHORIZED,
            "Invalid Authorization header format".to_string(),
        ));
    }

    // Extract the token
    let token = auth_header[7..].trim();

    // Verify the token
    match UserClaims::decode(token, &ACCESS_TOKEN_DECODE_KEY) {
        Ok(token_data) => {
            // Add user claims to request extensions for handlers to access
            let user_claims = token_data.claims;
            let mut request = request;
            request.extensions_mut().insert(user_claims);

            // Continue to the handler
            Ok(next.run(request).await)
        }
        Err(e) => {
            warn!("JWT token verification failed: {}", e);
            Err((StatusCode::UNAUTHORIZED, "Invalid token".to_string()))
        }
    }
}

/// Middleware for media endpoints that accepts JWT via Authorization header OR query parameter.
///
/// This is necessary because HTML5 `<video>` and `<img>` elements cannot send custom headers
/// when loading media via the `src` attribute. This middleware allows authentication via:
///
/// 1. `Authorization: Bearer <token>` header (preferred)
/// 2. `?token=<token>` query parameter (fallback for media elements)
///
/// # Security Note
///
/// Query parameter tokens are visible in browser history and server logs. This is standard
/// practice for media streaming but the frontend should be aware of this trade-off.
pub async fn media_auth_middleware(
    request: Request,
    next: Next,
) -> Result<Response, (StatusCode, String)> {
    debug!("Authenticating media request: {}", request.uri());

    // Try to extract token from Authorization header first
    let token = extract_token_from_header(&request).or_else(|| extract_token_from_query(&request));

    let token = token.ok_or_else(|| {
        warn!("No authentication token provided (checked header and query param)");
        (
            StatusCode::UNAUTHORIZED,
            "Authentication required. Provide Authorization header or token query parameter."
                .to_string(),
        )
    })?;

    // Verify the token
    match UserClaims::decode(&token, &ACCESS_TOKEN_DECODE_KEY) {
        Ok(token_data) => {
            // Add user claims to request extensions for handlers to access
            let user_claims = token_data.claims;
            let mut request = request;
            request.extensions_mut().insert(user_claims);

            // Continue to the handler
            Ok(next.run(request).await)
        }
        Err(e) => {
            warn!("JWT token verification failed: {}", e);
            Err((StatusCode::UNAUTHORIZED, "Invalid token".to_string()))
        }
    }
}

/// Extract Bearer token from Authorization header
fn extract_token_from_header(request: &Request) -> Option<String> {
    request
        .headers()
        .get("Authorization")
        .and_then(|header| header.to_str().ok())
        .filter(|auth| auth.starts_with("Bearer "))
        .map(|auth| auth[7..].trim().to_string())
}

/// Extract token from query parameter
fn extract_token_from_query(request: &Request) -> Option<String> {
    request.uri().query().and_then(|query| {
        // Parse query string to find 'token' parameter
        query.split('&').find_map(|pair| {
            let (key, value) = pair.split_once('=')?;
            if key == "token" {
                // URL decode the token value
                percent_decode(value)
            } else {
                None
            }
        })
    })
}

/// Simple percent-decoding for URL query parameters
fn percent_decode(input: &str) -> Option<String> {
    let mut result = String::with_capacity(input.len());
    let mut chars = input.chars().peekable();

    while let Some(c) = chars.next() {
        if c == '%' {
            // Read two hex digits
            let hex: String = chars.by_ref().take(2).collect();
            if hex.len() == 2 {
                if let Ok(byte) = u8::from_str_radix(&hex, 16) {
                    result.push(byte as char);
                } else {
                    return None; // Invalid hex
                }
            } else {
                return None; // Incomplete escape sequence
            }
        } else if c == '+' {
            result.push(' ');
        } else {
            result.push(c);
        }
    }

    Some(result)
}
