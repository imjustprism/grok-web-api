use figment::Figment;
use figment::providers::{Env, Format, Serialized, Toml};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub host: String,
    pub port: u16,
    #[serde(default)]
    pub api_key: Option<String>,
    pub grok_sso_cookie: String,
    pub grok_sso_rw_cookie: String,
    #[serde(default)]
    pub grok_extra_cookies: Option<String>,
    #[serde(default)]
    pub token_provider_url: Option<String>,
    #[serde(default)]
    pub challenge_header_hex: Option<String>,
    #[serde(default)]
    pub challenge_suffix: Option<String>,
    #[serde(default)]
    pub challenge_trailer: Option<u8>,
    pub grok_base_url: String,
    pub log_level: String,
    pub session_check_interval_secs: u64,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            host: "0.0.0.0".into(),
            port: 3000,
            api_key: None,
            grok_sso_cookie: String::new(),
            grok_sso_rw_cookie: String::new(),
            grok_extra_cookies: None,
            token_provider_url: None,
            challenge_header_hex: None,
            challenge_suffix: None,
            challenge_trailer: None,
            grok_base_url: "https://grok.com".into(),
            log_level: "info".into(),
            session_check_interval_secs: 300,
        }
    }
}

impl Config {
    #[allow(clippy::result_large_err)]
    pub fn load() -> figment::error::Result<Self> {
        Figment::from(Serialized::defaults(Self::default()))
            .merge(Toml::file("config.toml"))
            .merge(Env::prefixed("GROK_API_").split("__"))
            .merge(
                Env::raw()
                    .only(&[
                        "GROK_SSO_COOKIE",
                        "GROK_SSO_RW_COOKIE",
                        "GROK_EXTRA_COOKIES",
                        "TOKEN_PROVIDER_URL",
                        "CHALLENGE_HEADER_HEX",
                        "CHALLENGE_SUFFIX",
                        "CHALLENGE_TRAILER",
                        "API_KEY",
                        "HOST",
                        "PORT",
                        "LOG_LEVEL",
                    ])
                    .map(|key| match key.as_str() {
                        "GROK_SSO_COOKIE" => "grok_sso_cookie".into(),
                        "GROK_SSO_RW_COOKIE" => "grok_sso_rw_cookie".into(),
                        "GROK_EXTRA_COOKIES" => "grok_extra_cookies".into(),
                        "TOKEN_PROVIDER_URL" => "token_provider_url".into(),
                        "CHALLENGE_HEADER_HEX" => "challenge_header_hex".into(),
                        "CHALLENGE_SUFFIX" => "challenge_suffix".into(),
                        "CHALLENGE_TRAILER" => "challenge_trailer".into(),
                        "API_KEY" => "api_key".into(),
                        "HOST" => "host".into(),
                        "PORT" => "port".into(),
                        "LOG_LEVEL" => "log_level".into(),
                        other => other.into(),
                    }),
            )
            .extract()
    }
}
