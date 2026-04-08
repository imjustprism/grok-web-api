use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

#[derive(Debug, Clone)]
pub struct GrokAuth {
    cookies: String,
    session_valid: Arc<AtomicBool>,
}

impl GrokAuth {
    pub fn new(sso: impl Into<String>, sso_rw: impl Into<String>) -> crate::error::Result<Self> {
        Self::with_extra_cookies(sso, sso_rw, "")
    }

    pub fn with_extra_cookies(
        sso: impl Into<String>,
        sso_rw: impl Into<String>,
        extra: impl Into<String>,
    ) -> crate::error::Result<Self> {
        let sso = sso.into();
        let sso_rw = sso_rw.into();
        let extra = extra.into();

        if sso.is_empty() {
            return Err(crate::error::GrokError::Config(
                "GROK_SSO_COOKIE must not be empty".into(),
            ));
        }
        if sso_rw.is_empty() {
            return Err(crate::error::GrokError::Config(
                "GROK_SSO_RW_COOKIE must not be empty".into(),
            ));
        }

        let mut cookies = format!("sso={sso}; sso-rw={sso_rw}");
        if !extra.is_empty() {
            cookies.push_str("; ");
            cookies.push_str(&extra);
        }

        Ok(Self {
            cookies,
            session_valid: Arc::new(AtomicBool::new(true)),
        })
    }

    #[must_use]
    pub fn cookie_header(&self) -> &str {
        &self.cookies
    }

    #[must_use]
    pub fn is_valid(&self) -> bool {
        self.session_valid.load(Ordering::Relaxed)
    }

    pub fn invalidate(&self) {
        self.session_valid.store(false, Ordering::Relaxed);
    }

    pub fn revalidate(&self) {
        self.session_valid.store(true, Ordering::Relaxed);
    }
}
