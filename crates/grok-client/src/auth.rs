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
        let sso_rw_raw = sso_rw.into();
        let extra = extra.into();

        if sso.is_empty() {
            return Err(crate::error::GrokError::Config(
                "GROK_SSO_COOKIE must not be empty".into(),
            ));
        }

        let sso_rw = if sso_rw_raw.is_empty() {
            sso.as_str()
        } else {
            sso_rw_raw.as_str()
        };

        if sso.contains(['\r', '\n', '\0'])
            || sso_rw.contains(['\r', '\n', '\0'])
            || extra.contains(['\r', '\n', '\0'])
        {
            return Err(crate::error::GrokError::Config(
                "cookie values contain CR/LF/null — likely paste error".into(),
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_sso_rejected() {
        assert!(GrokAuth::new("", "rw").is_err());
    }

    #[test]
    fn empty_sso_rw_defaults_to_sso() {
        let auth = GrokAuth::new("abc", "").unwrap();
        assert_eq!(auth.cookie_header(), "sso=abc; sso-rw=abc");
    }

    #[test]
    fn distinct_sso_rw_preserved() {
        let auth = GrokAuth::new("a", "b").unwrap();
        assert_eq!(auth.cookie_header(), "sso=a; sso-rw=b");
    }

    #[test]
    fn extra_cookies_appended() {
        let auth = GrokAuth::with_extra_cookies("a", "b", "k=v; k2=v2").unwrap();
        assert_eq!(auth.cookie_header(), "sso=a; sso-rw=b; k=v; k2=v2");
    }

    #[test]
    fn rejects_crlf_in_cookies() {
        assert!(GrokAuth::new("bad\r\nvalue", "ok").is_err());
        assert!(GrokAuth::new("ok", "bad\nvalue").is_err());
        assert!(GrokAuth::new("ok\0", "ok").is_err());
    }

    #[test]
    fn validity_state_transitions() {
        let auth = GrokAuth::new("a", "b").unwrap();
        assert!(auth.is_valid());
        auth.invalidate();
        assert!(!auth.is_valid());
        auth.revalidate();
        assert!(auth.is_valid());
    }
}
