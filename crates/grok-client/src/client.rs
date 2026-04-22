use std::sync::Arc;
use std::time::Duration;
use tracing::{debug, warn};
use wreq::header::{CONTENT_TYPE, COOKIE, HeaderMap, HeaderValue};
use wreq::{Client, Response, StatusCode};
use wreq_util::Emulation;

use crate::auth::GrokAuth;
use crate::error::{GrokError, RateLimitType, Result};

const DEFAULT_BASE_URL: &str = "https://grok.com";
const DEFAULT_TIMEOUT: Duration = Duration::from_secs(120);
const STREAM_TIMEOUT: Duration = Duration::from_secs(300);

#[derive(Debug, Clone)]
pub struct GrokClient {
    http: Client,
    base_url: String,
    api_prefix: String,
    static_headers: HeaderMap,
    auth: GrokAuth,
    token_provider: Option<Arc<dyn TokenProvider>>,
}

pub trait TokenProvider: Send + Sync + std::fmt::Debug {
    fn generate(
        &self,
        path: &str,
        method: &str,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<TokenPair>> + Send + '_>>;
}

#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct TokenPair {
    pub statsig_id: String,
    pub request_id: String,
}

impl TokenPair {
    #[must_use]
    pub fn fallback() -> Self {
        Self {
            statsig_id: String::new(),
            request_id: uuid::Uuid::new_v4().to_string(),
        }
    }
}

#[derive(Debug)]
pub struct HttpTokenProvider {
    client: wreq::Client,
    url: String,
}

impl HttpTokenProvider {
    pub fn new(url: impl Into<String>) -> Result<Self> {
        let client = wreq::Client::builder()
            .timeout(Duration::from_secs(5))
            .build()
            .map_err(GrokError::Request)?;
        Ok(Self {
            client,
            url: url.into(),
        })
    }
}

impl TokenProvider for HttpTokenProvider {
    fn generate(
        &self,
        path: &str,
        method: &str,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<TokenPair>> + Send + '_>> {
        let path = path.to_owned();
        let method = method.to_owned();
        Box::pin(async move {
            let resp = self
                .client
                .post(&self.url)
                .json(&serde_json::json!({ "path": path, "method": method }))
                .send()
                .await
                .map_err(GrokError::Request)?;

            let body: serde_json::Value = resp.json().await.map_err(GrokError::Request)?;
            let statsig_id = body["x-statsig-id"].as_str().unwrap_or("").to_owned();
            if statsig_id.is_empty() {
                warn!("token provider returned empty x-statsig-id — anti-bot will reject");
            }
            Ok(TokenPair {
                statsig_id,
                request_id: body["x-xai-request-id"]
                    .as_str()
                    .map(str::to_owned)
                    .unwrap_or_else(|| uuid::Uuid::new_v4().to_string()),
            })
        })
    }
}

impl GrokClient {
    pub fn new(auth: GrokAuth) -> Result<Self> {
        Self::with_base_url(auth, DEFAULT_BASE_URL)
    }

    pub fn with_base_url(auth: GrokAuth, base_url: impl Into<String>) -> Result<Self> {
        let http = Client::builder()
            .emulation(Emulation::Chrome136)
            .timeout(DEFAULT_TIMEOUT)
            .connect_timeout(Duration::from_secs(10))
            .pool_max_idle_per_host(10)
            .gzip(true)
            .build()
            .map_err(GrokError::Request)?;

        let base = base_url.into().trim_end_matches('/').to_owned();
        let api_prefix = format!("{base}/rest/app-chat");

        let mut static_headers = HeaderMap::with_capacity(8);
        let cookie_val = HeaderValue::from_str(auth.cookie_header()).map_err(|e| {
            GrokError::Config(format!(
                "cookie header contains invalid bytes (check for CR/LF/null in env): {e}"
            ))
        })?;
        let origin_val = HeaderValue::from_str(&base)
            .map_err(|e| GrokError::Config(format!("base_url is not a valid header value: {e}")))?;
        let referer_val = HeaderValue::from_str(&format!("{base}/"))
            .map_err(|e| GrokError::Config(format!("base_url is not a valid header value: {e}")))?;
        static_headers.insert(COOKIE, cookie_val);
        static_headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
        static_headers.insert("accept", HeaderValue::from_static("*/*"));
        static_headers.insert("origin", origin_val);
        static_headers.insert("referer", referer_val);
        static_headers.insert("sec-fetch-dest", HeaderValue::from_static("empty"));
        static_headers.insert("sec-fetch-mode", HeaderValue::from_static("cors"));
        static_headers.insert("sec-fetch-site", HeaderValue::from_static("same-origin"));

        Ok(Self {
            http,
            base_url: base,
            api_prefix,
            static_headers,
            auth,
            token_provider: None,
        })
    }

    #[must_use]
    pub fn with_token_provider(mut self, provider: impl TokenProvider + 'static) -> Self {
        self.token_provider = Some(Arc::new(provider));
        self
    }

    #[must_use]
    pub fn auth(&self) -> &GrokAuth {
        &self.auth
    }

    #[must_use]
    pub fn url(&self, path: &str) -> String {
        format!("{}/{}", self.api_prefix, path.trim_start_matches('/'))
    }

    async fn build_headers(&self, path: &str, method: &str) -> HeaderMap {
        let mut headers = self.static_headers.clone();

        let pair = match self.token_provider {
            Some(ref provider) => match provider.generate(path, method).await {
                Ok(pair) => {
                    debug!(
                        "Token provider generated statsig_id ({} chars)",
                        pair.statsig_id.len()
                    );
                    pair
                }
                Err(e) => {
                    warn!("Token provider failed, using fallback: {e}");
                    TokenPair::fallback()
                }
            },
            None => TokenPair::fallback(),
        };

        match HeaderValue::from_str(&pair.request_id) {
            Ok(val) => {
                headers.insert("x-xai-request-id", val);
            }
            Err(e) => warn!("invalid request_id header value, skipping: {e}"),
        }
        match HeaderValue::from_str(&pair.statsig_id) {
            Ok(val) => {
                headers.insert("x-statsig-id", val);
            }
            Err(e) => warn!("invalid statsig_id header value, skipping: {e}"),
        }

        headers
    }

    async fn request(&self, method: wreq::Method, path: &str) -> Result<wreq::RequestBuilder> {
        let trimmed = path.trim_start_matches('/');
        let path_only = trimmed.split_once('?').map_or(trimmed, |(p, _)| p);
        let challenge_path = format!("/rest/app-chat/{path_only}");
        let url = format!("{}/rest/app-chat/{trimmed}", self.base_url);
        let headers = self.build_headers(&challenge_path, method.as_str()).await;
        Ok(self.http.request(method, &url).headers(headers))
    }

    #[inline]
    async fn send(&self, builder: wreq::RequestBuilder) -> Result<Response> {
        let response = builder.send().await.map_err(GrokError::Request)?;
        self.check_response(response).await
    }

    pub async fn post_base<B: serde::Serialize>(&self, path: &str, body: &B) -> Result<Response> {
        let trimmed = path.trim_start_matches('/');
        let url = format!("{}/{}", self.base_url, trimmed);
        let headers = self.build_headers(&format!("/{trimmed}"), "POST").await;
        let response = self
            .http
            .post(&url)
            .headers(headers)
            .json(body)
            .send()
            .await
            .map_err(GrokError::Request)?;
        self.check_response(response).await
    }

    pub async fn get(&self, path: &str) -> Result<Response> {
        let rb = self.request(wreq::Method::GET, path).await?;
        self.send(rb).await
    }

    pub async fn get_with_query<Q: serde::Serialize + ?Sized>(
        &self,
        path: &str,
        query: &Q,
    ) -> Result<Response> {
        let rb = self.request(wreq::Method::GET, path).await?.query(query);
        self.send(rb).await
    }

    pub async fn post<B: serde::Serialize>(&self, path: &str, body: &B) -> Result<Response> {
        let rb = self.request(wreq::Method::POST, path).await?.json(body);
        self.send(rb).await
    }

    pub async fn post_stream<B: serde::Serialize>(&self, path: &str, body: &B) -> Result<Response> {
        let rb = self
            .request(wreq::Method::POST, path)
            .await?
            .timeout(STREAM_TIMEOUT)
            .json(body);
        self.send(rb).await
    }

    pub async fn put<B: serde::Serialize>(&self, path: &str, body: &B) -> Result<Response> {
        let rb = self.request(wreq::Method::PUT, path).await?.json(body);
        self.send(rb).await
    }

    pub async fn delete(&self, path: &str) -> Result<Response> {
        let rb = self.request(wreq::Method::DELETE, path).await?;
        self.send(rb).await
    }

    pub async fn delete_with_query<Q: serde::Serialize + ?Sized>(
        &self,
        path: &str,
        query: &Q,
    ) -> Result<Response> {
        let rb = self.request(wreq::Method::DELETE, path).await?.query(query);
        self.send(rb).await
    }

    pub async fn get_json<T: serde::de::DeserializeOwned>(&self, path: &str) -> Result<T> {
        let response = self.get(path).await?;
        self.json(response).await
    }

    pub async fn get_query_json<T: serde::de::DeserializeOwned, Q: serde::Serialize + ?Sized>(
        &self,
        path: &str,
        query: &Q,
    ) -> Result<T> {
        let response = self.get_with_query(path, query).await?;
        self.json(response).await
    }

    pub async fn post_json<T: serde::de::DeserializeOwned, B: serde::Serialize>(
        &self,
        path: &str,
        body: &B,
    ) -> Result<T> {
        let response = self.post(path, body).await?;
        self.json(response).await
    }

    pub async fn put_json<T: serde::de::DeserializeOwned, B: serde::Serialize>(
        &self,
        path: &str,
        body: &B,
    ) -> Result<T> {
        let response = self.put(path, body).await?;
        self.json(response).await
    }

    #[inline]
    async fn check_response(&self, response: Response) -> Result<Response> {
        let status = response.status();

        match status {
            s if s.is_success() => {
                self.auth.revalidate();
                Ok(response)
            }
            StatusCode::UNAUTHORIZED | StatusCode::FORBIDDEN => {
                let body = response.text().await.unwrap_or_default();
                warn!("Grok rejected request (HTTP {status}): {body}");
                if body.contains("anti-bot") {
                    Err(GrokError::Upstream {
                        status: status.as_u16(),
                        body,
                    })
                } else {
                    self.auth.invalidate();
                    Err(GrokError::AuthExpired)
                }
            }
            StatusCode::TOO_MANY_REQUESTS => {
                let retry_after = response
                    .headers()
                    .get("retry-after")
                    .and_then(|v| v.to_str().ok())
                    .and_then(|v| v.parse::<u64>().ok());
                let body = response.text().await.unwrap_or_default();
                Err(GrokError::RateLimited {
                    message: body,
                    wait_seconds: retry_after,
                    limit_type: RateLimitType::User,
                })
            }
            StatusCode::NOT_FOUND => {
                let body = response.text().await.unwrap_or_default();
                Err(GrokError::NotFound(body))
            }
            _ => {
                let body = response.text().await.unwrap_or_default();
                Err(GrokError::Upstream {
                    status: status.as_u16(),
                    body,
                })
            }
        }
    }

    pub async fn json<T: serde::de::DeserializeOwned>(&self, response: Response) -> Result<T> {
        response.json().await.map_err(GrokError::Request)
    }

    pub async fn check_session(&self) -> Result<bool> {
        #[derive(serde::Serialize)]
        struct HealthQuery {
            #[serde(rename = "pageSize")]
            page_size: u32,
        }

        match self
            .get_with_query("conversations", &HealthQuery { page_size: 1 })
            .await
        {
            Ok(_) => Ok(true),
            Err(GrokError::AuthExpired) => Ok(false),
            Err(e) => Err(e),
        }
    }
}
