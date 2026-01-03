use self::state::AppState;
use crate::configure::server::AcmeChallenge;
use crate::configure::AppConfig;
use crate::error::AppResult;
use crate::routes::create_router_app;
use futures::StreamExt;
use rustls_acme::caches::DirCache;
use rustls_acme::{AcmeConfig, UseChallenge};
pub mod state;

pub struct AppServer {
  pub state: AppState,
}
impl AppServer {
  pub async fn new(config: AppConfig) -> AppResult<Self> {
    let state = AppState::new(config).await?;
    Ok(Self { state })
  }

  pub async fn run(self) -> AppResult<()> {
    let config = self.state.config.clone();
    let addr = config.server.get_socket_addr()?;
    let tls_config = config.server.tls.clone();
    let acme_config = config.server.acme.clone();
    let router = create_router_app(self.state);

    if let Some(acme) = acme_config.filter(|acme| acme.enabled) {
      if tls_config.as_ref().is_some_and(|tls| tls.enabled) {
        return Err(crate::error::AppError::InvalidPayloadError(
          "server.tls.enabled cannot be true when server.acme.enabled is true".to_string(),
        ));
      }

      let domains: Vec<String> = acme
        .domains
        .iter()
        .filter_map(|domain| {
          let trimmed = domain.trim();
          if trimmed.is_empty() {
            None
          } else {
            Some(trimmed.to_string())
          }
        })
        .collect();
      if domains.is_empty() {
        return Err(crate::error::AppError::InvalidPayloadError(
          "server.acme.domains must include at least one domain".to_string(),
        ));
      }
      if acme.production && acme.cache_dir.is_none() {
        tracing::warn!("server.acme.cache_dir is unset; production ACME should persist cache to avoid rate limits");
      }

      let contacts = acme
        .contact_emails
        .iter()
        .filter_map(|email| {
          let trimmed = email.trim();
          if trimmed.is_empty() {
            None
          } else if trimmed.starts_with("mailto:") {
            Some(trimmed.to_string())
          } else {
            Some(format!("mailto:{trimmed}"))
          }
        })
        .collect::<Vec<_>>();

      let mut state = AcmeConfig::new(domains)
        .contact(contacts)
        .cache_option(acme.cache_dir.as_ref().map(|dir| DirCache::new(dir.clone())))
        .directory_lets_encrypt(acme.production)
        .challenge_type(match acme.challenge {
          AcmeChallenge::TlsAlpn01 => UseChallenge::TlsAlpn01,
          AcmeChallenge::Http01 => UseChallenge::Http01,
        })
        .state();

      let acceptor = state.axum_acceptor(state.default_rustls_config());
      let http01_service = match acme.challenge {
        AcmeChallenge::Http01 => Some(state.http01_challenge_tower_service()),
        AcmeChallenge::TlsAlpn01 => None,
      };

      tokio::spawn(async move {
        while let Some(event) = state.next().await {
          match event {
            Ok(ok) => tracing::info!("acme event: {:?}", ok),
            Err(err) => tracing::error!("acme error: {:?}", err),
          }
        }
      });

      tracing::info!("Starting HTTPS server (ACME) on: {addr}");
      if let Some(challenge_service) = http01_service {
        let http_port = acme.http_port.unwrap_or(80);
        let http_addr = format!("{}:{}", config.server.addr, http_port).parse()?;
        tracing::info!("Starting HTTP-01 challenge listener on: {http_addr}");
        let http_router = axum::Router::new()
          .route_service("/.well-known/acme-challenge/{challenge_token}", challenge_service);
        let http_server = axum_server::bind(http_addr).serve(http_router.into_make_service());
        let https_server = axum_server::bind(addr)
          .acceptor(acceptor)
          .serve(router.into_make_service());
        tokio::try_join!(https_server, http_server)?;
      } else {
        axum_server::bind(addr)
          .acceptor(acceptor)
          .serve(router.into_make_service())
          .await?;
      }
      return Ok(());
    }

    if let Some(tls) = tls_config.filter(|tls| tls.enabled) {
      let cert_path = tls.cert_path.as_ref().ok_or_else(|| {
        crate::error::AppError::InvalidPayloadError("server.tls.cert_path is required".to_string())
      })?;
      let key_path = tls.key_path.as_ref().ok_or_else(|| {
        crate::error::AppError::InvalidPayloadError("server.tls.key_path is required".to_string())
      })?;
      tracing::info!("Starting HTTPS server on: {addr}");
      let tls = axum_server::tls_rustls::RustlsConfig::from_pem_file(cert_path, key_path).await?;
      axum_server::bind_rustls(addr, tls)
        .serve(router.into_make_service())
        .await?;
    } else {
      tracing::info!("Starting HTTP server on: {addr}");
      let tcp = tokio::net::TcpListener::bind(addr).await?;
      axum::serve(tcp, router).await?;
    }
    Ok(())
  }
}
