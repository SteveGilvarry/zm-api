use axum::{extract::Request, http::StatusCode, middleware::Next, response::Response};
use tracing::{info, warn};

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
