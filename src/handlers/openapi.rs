use utoipa::{
    openapi::security::{Http, HttpAuthScheme, SecurityScheme},
    Modify, OpenApi,
};

use crate::dto::*;
use crate::dto::wrappers::*;
use crate::error::{AppError, AppResponseError};
use crate::util::claim::UserClaims;

#[derive(OpenApi)]
#[openapi(
    info(
        version = crate::constant::API_VERSION,
        title = "Zoneminder API",
    ),
    paths(
        // auth api
        crate::handlers::auth::login,
        crate::handlers::auth::refresh_token,
        crate::handlers::auth::logout,
        
        // server api
        crate::handlers::server::health_check,
        crate::handlers::server::get_version,
        
        // monitor api
        crate::handlers::monitor::index,
        crate::handlers::monitor::view, 
        crate::handlers::monitor::create,
        crate::handlers::monitor::edit,
        crate::handlers::monitor::delete,
        crate::handlers::monitor::update_state,
        crate::handlers::monitor::alarm_control,
        
        // streaming api
        crate::handlers::streaming::register_stream,
        crate::handlers::streaming::get_stream,
        crate::handlers::streaming::delete_stream,
        
        // We'll enable events paths once the handlers are fixed
        crate::handlers::events::index,
        crate::handlers::events::get,
        crate::handlers::events::create,
        crate::handlers::events::update,
        crate::handlers::events::delete,
        crate::handlers::events::counts,
    ),
    components(
        schemas(
            // auth schemas
            LoginRequest,
            LoginResponse,
            AppResponseError,
            AppError,
            MessageResponse,
            TokenInfoRequest,
            UserClaims,
            TokenResponse,
            RefreshTokenRequest,
            
            // server schemas
            VersionResponse,
            ServiceStatusResponse,
            
            // monitor schemas
            CreateMonitorRequest,
            UpdateMonitorRequest,
            UpdateStateRequest,
            AlarmControlRequest,
            MonitorResponse,
            
            // streaming schemas
            crate::dto::response::StreamEndpoints,
            crate::dto::response::MonitorStreamingDetails,
            
            // events schemas
            crate::dto::request::events::EventCreateRequest,
            crate::dto::request::events::EventUpdateRequest,
            crate::dto::request::events::EventQueryParams,
            crate::dto::response::events::EventResponse,
            crate::dto::response::events::PaginatedEventsResponse,
            crate::dto::response::events::EventCountResponse,
            crate::dto::response::events::EventCountsResponse,
            
            // wrapper types for OpenAPI schema
            DateTimeWrapper,
            NaiveDateTimeWrapper,
            DecimalWrapper,
            SchemeWrapper
        )
    ),
    tags(
        (name = "Auth", description = "Authentication endpoints"),
        (name = "Server", description = "Server information endpoints"),
        (name = "Monitors", description = "Monitor management endpoints"),
        (name = "Streaming", description = "Video streaming endpoints"),
        (name = "Events", description = "Event management endpoints")
    ),
    modifiers(&SecurityAddon)
)]
pub struct ApiDoc;

struct SecurityAddon;

impl Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        let components = openapi.components.as_mut().unwrap();
        components.add_security_scheme(
            "jwt",
            SecurityScheme::Http(Http::new(HttpAuthScheme::Bearer)),
        )
    }
}