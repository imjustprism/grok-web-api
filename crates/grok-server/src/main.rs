use std::net::SocketAddr;
use std::time::Duration;
use tokio::net::TcpListener;
use tower_http::compression::CompressionLayer;
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;
use tracing::{debug, warn};

mod auth;
mod config;
mod error;
mod routes;
mod state;

use config::Config;
use state::AppState;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let _ = dotenvy::dotenv();

    let config = Config::load().map_err(|e| anyhow::anyhow!("Failed to load config: {e}"))?;

    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| config.log_level.parse().unwrap_or_else(|_| "info".into())),
        )
        .init();

    debug!("Starting grok-web-api server");

    if config.grok_sso_cookie.is_empty() {
        eprintln!("ERROR: GROK_SSO_COOKIE is not set.");
        eprintln!("  1. Log in to grok.com in your browser");
        eprintln!("  2. Open DevTools (F12) > Application > Cookies > grok.com");
        eprintln!("  3. Copy the value of 'sso' and 'sso-rw' cookies");
        eprintln!("  4. Set GROK_SSO_COOKIE and GROK_SSO_RW_COOKIE environment variables");
        std::process::exit(1);
    }

    if config.challenge_header_hex.is_none() && config.token_provider_url.is_none() {
        eprintln!("WARNING: No challenge config or token provider set. POST requests will be rejected by anti-bot.");
        eprintln!("  Extract values: curl http://localhost:{}/setup", config.port);
    }

    let grok_auth = grok_client::GrokAuth::with_extra_cookies(
        &config.grok_sso_cookie,
        &config.grok_sso_rw_cookie,
        config.grok_extra_cookies.as_deref().unwrap_or(""),
    )
    .map_err(|e| anyhow::anyhow!("{e}"))?;

    let mut grok_client = if config.grok_base_url != "https://grok.com" {
        debug!(base_url = %config.grok_base_url, "Using custom Grok base URL");
        grok_client::GrokClient::with_base_url(grok_auth, &config.grok_base_url)
    } else {
        grok_client::GrokClient::new(grok_auth)
    }
    .map_err(|e| anyhow::anyhow!("{e}"))?;

    {
        let header = config.challenge_header_hex.clone();
        let suffix = config.challenge_suffix.clone();
        if let (Some(h), Some(s)) = (header, suffix) {
            let trailer = config.challenge_trailer.unwrap_or(3);
            let challenge = grok_client::ChallengeConfig::new(&h, &s, trailer)
                .map_err(|e| anyhow::anyhow!("{e}"))?;
            grok_client = grok_client.with_token_provider(challenge);
            debug!("Challenge token generator enabled");
        }
    }

    if let Some(ref token_url) = config.token_provider_url {
        debug!(url = %token_url, "Using external token provider");
        let provider = grok_client::HttpTokenProvider::new(token_url)
            .map_err(|e| anyhow::anyhow!("{e}"))?;
        grok_client = grok_client.with_token_provider(provider);
    }

    match grok_client.check_session().await {
        Ok(true) => debug!("Grok session is valid"),
        Ok(false) => {
            warn!("Grok session expired, update cookies")
        }
        Err(e) => warn!("Could not validate Grok session: {e}"),
    }

    let state = AppState::new(grok_client, config.clone());

    let health_state = state.clone();
    let check_interval = Duration::from_secs(config.session_check_interval_secs);
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(check_interval);
        loop {
            interval.tick().await;
            match health_state.client.check_session().await {
                Ok(true) => tracing::debug!("Session health check: valid"),
                Ok(false) => warn!("Session health check: expired"),
                Err(e) => warn!("Session health check failed: {e}"),
            }
        }
    });

    let app = routes::router(state)
        .layer(CompressionLayer::new())
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http());

    let addr: SocketAddr = format!("{}:{}", config.host, config.port)
        .parse()
        .map_err(|e| anyhow::anyhow!("Invalid bind address: {e}"))?;

    debug!(%addr, "Listening");

    let listener = TcpListener::bind(addr).await?;
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    debug!("Server shut down");
    Ok(())
}

async fn shutdown_signal() {
    let ctrl_c = tokio::signal::ctrl_c();

    #[cfg(unix)]
    {
        let mut sigterm =
            tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
                .expect("sigterm handler failed");
        tokio::select! {
            _ = ctrl_c => {},
            _ = sigterm.recv() => {},
        }
    }

    #[cfg(not(unix))]
    {
        ctrl_c.await.expect("ctrl+c handler failed");
    }
}
