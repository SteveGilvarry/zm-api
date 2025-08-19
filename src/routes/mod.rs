use axum::{Router, routing::any, http::{Method, HeaderName}};
use utoipa_swagger_ui::SwaggerUi;
use utoipa::OpenApi;
use axum::extract::MatchedPath;
use tower_http::cors::CorsLayer;
use crate::handlers::openapi::ApiDoc;
use crate::server::state::AppState;

pub mod server;
pub mod auth;
pub mod monitors;
pub mod streaming;
pub mod events; // Add events module
pub mod mse; // Add MSE module

async fn fallback_handler(path: MatchedPath) -> &'static str {
    tracing::error!("Unknown route: {}", path.as_str());
    "Unknown route"
}

pub fn create_router_app(state: AppState) -> Router {
    // Get frontend URL from environment variable or use default localhost addresses
    let frontend_urls = std::env::var("ALLOWED_ORIGINS")
        .unwrap_or_else(|_| "http://localhost:3000,http://localhost:5173,http://localhost:8000".to_string());
    
    // Parse the URLs into a Vec of HeaderValues for CORS configuration
    let origins = frontend_urls
        .split(',')
        .filter_map(|origin| origin.parse().ok())
        .collect::<Vec<_>>();
    
    tracing::info!("Configuring CORS with allowed origins: {:?}", frontend_urls);
    
    // Configure CORS to allow requests from the frontend(s)
    let cors = CorsLayer::new()
        // Allow frontend origins to access the API
        .allow_origin(origins)
        // Allow common HTTP methods needed for a RESTful API
        .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE, Method::PATCH, Method::OPTIONS])
        // Allow common HTTP headers used in API requests
        .allow_headers([
            HeaderName::from_static("authorization"),
            HeaderName::from_static("content-type"),
            HeaderName::from_static("x-requested-with"),
            HeaderName::from_static("accept"),
            HeaderName::from_static("origin"),
        ])
        .allow_credentials(true);

    let server_routes = server::add_server_routes(Router::new());
    let auth_routes = auth::add_routers(Router::new());
    let monitors_routes = monitors::add_monitor_routes(Router::new());
    let streaming_routes = streaming::add_streaming_routes(Router::new());
    let events_routes = events::add_event_routes(Router::new()); // Add events routes
    let mse_routes = mse::add_mse_routes(Router::new()); // Add MSE routes

    Router::new()
        .merge(
            SwaggerUi::new("/swagger-ui")
                .url("/api-docs/openapi.json", ApiDoc::openapi())
        )
        .merge(server_routes)
        .merge(auth_routes)
        .merge(monitors_routes)
        .merge(streaming_routes)
        .merge(events_routes) // Merge events routes
        .merge(mse_routes) // Merge MSE routes
        .fallback(any(fallback_handler))
        .layer(cors)  // Apply CORS middleware to all routes
        .with_state(state)
}