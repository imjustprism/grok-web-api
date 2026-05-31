use figment::Figment;
use figment::providers::{Env, Format, Serialized, Toml};
use serde::{Deserialize, Serialize};

pub(crate) const ENV_KEYS: &[&str] = &[
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
    "SESSION_CHECK_INTERVAL_SECS",
    "GROK_BASE_URL",
];

const DEFAULT_PORT: u16 = 3000;
const DEFAULT_SESSION_CHECK_SECS: u64 = 300;
const DEFAULT_HOST: &str = "0.0.0.0";
const DEFAULT_BASE_URL: &str = "https://grok.com";

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
            host: DEFAULT_HOST.into(),
            port: DEFAULT_PORT,
            api_key: None,
            grok_sso_cookie: String::new(),
            grok_sso_rw_cookie: String::new(),
            grok_extra_cookies: None,
            token_provider_url: None,
            challenge_header_hex: None,
            challenge_suffix: None,
            challenge_trailer: None,
            grok_base_url: DEFAULT_BASE_URL.into(),
            log_level: "info".into(),
            session_check_interval_secs: DEFAULT_SESSION_CHECK_SECS,
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
                    .only(ENV_KEYS)
                    .map(|key| key.as_str().to_ascii_lowercase().into()),
            )
            .extract()
    }
}
