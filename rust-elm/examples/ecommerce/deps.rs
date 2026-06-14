//! Shop dependencies — live vs mock implementations registered on [`Environment`].
//!
//! Mirrors the httpbin + chrono + WebSocket pattern: swap `Environment::new().with(...)` to
//! test without network calls.

use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

use chrono::{DateTime, Local, TimeZone};
use rust_elm::dependencies::{DependencyError, DependencyKey, DependencyValues};
use rust_elm::Environment;
use serde::{Deserialize, Serialize};

pub const HTTPBIN_GET: &str = "https://httpbin.org/get";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HttpBin {
    pub origin: String,
}

/// HTTP client dependency (production uses reqwest; mock returns a fixed origin).
pub trait HttpRequestDependency: Send + Sync {
    fn get_request(
        &self,
        url: String,
    ) -> Pin<Box<dyn Future<Output = Result<HttpBin, String>> + Send + '_>>;
}

/// Wall-clock dependency (production uses `Local::now`; mock uses a fixed instant).
pub trait DateDependency: Send + Sync {
    fn current(&self) -> Pin<Box<dyn Future<Output = DateTime<Local>> + Send + '_>>;
}

/// Checkout payment-status WebSocket URL (used by `Sub::websocket` in subscriptions).
pub trait WebSocketDependency: Send + Sync {
    fn checkout_status_url(&self) -> &'static str;
}

#[derive(Clone)]
pub struct HttpDep(pub Arc<dyn HttpRequestDependency>);

#[derive(Clone)]
pub struct DateDep(pub Arc<dyn DateDependency>);

#[derive(Clone)]
pub struct WebDep(pub Arc<dyn WebSocketDependency>);

// ── Live implementations ───────────────────────────────────────────────────

pub struct HttpBinServiceLive {
    client: reqwest::Client,
}

impl HttpBinServiceLive {
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::new(),
        }
    }
}

impl HttpRequestDependency for HttpBinServiceLive {
    fn get_request(
        &self,
        url: String,
    ) -> Pin<Box<dyn Future<Output = Result<HttpBin, String>> + Send + '_>> {
        let client = self.client.clone();
        Box::pin(async move {
            let response = client
                .get(url.as_str())
                .send()
                .await
                .map_err(|e| e.to_string())?
                .json::<serde_json::Value>()
                .await
                .map_err(|e| e.to_string())?;
            let bin = HttpBin {
                origin: response["origin"]
                    .as_str()
                    .unwrap_or("unknown")
                    .to_string(),
            };
            println!("prod http = {bin:?}");
            Ok(bin)
        })
    }
}

pub struct DateServiceLive;

impl DateDependency for DateServiceLive {
    fn current(&self) -> Pin<Box<dyn Future<Output = DateTime<Local>> + Send + '_>> {
        Box::pin(async { Local::now() })
    }
}

pub struct CheckoutWebSocketLive;

impl WebSocketDependency for CheckoutWebSocketLive {
    fn checkout_status_url(&self) -> &'static str {
        "wss://echo.websocket.org"
    }
}

// ── Mock implementations ─────────────────────────────────────────────────────

pub struct HttpBinServiceMock;

impl HttpRequestDependency for HttpBinServiceMock {
    fn get_request(
        &self,
        _url: String,
    ) -> Pin<Box<dyn Future<Output = Result<HttpBin, String>> + Send + '_>> {
        Box::pin(async move {
            let bin = HttpBin {
                origin: "mock".to_string(),
            };
            println!("mock http = {bin:?}");
            Ok(bin)
        })
    }
}

pub struct DateServiceMock;

impl DateDependency for DateServiceMock {
    fn current(&self) -> Pin<Box<dyn Future<Output = DateTime<Local>> + Send + '_>> {
        Box::pin(async {
            Local
                .timestamp_opt(557_152_051, 0)
                .single()
                .expect("valid mock timestamp")
        })
    }
}

pub struct CheckoutWebSocketMock;

impl WebSocketDependency for CheckoutWebSocketMock {
    fn checkout_status_url(&self) -> &'static str {
        "mock://checkout/status"
    }
}

// ── DependencyKey registration (rust_dependencies pattern) ─────────────────

pub struct HttpDepKey;

impl DependencyKey for HttpDepKey {
    type Value = HttpDep;

    fn live() -> Self::Value {
        HttpDep(Arc::new(HttpBinServiceLive::new()))
    }

    fn try_test() -> Result<Self::Value, DependencyError> {
        Ok(HttpDep(Arc::new(HttpBinServiceMock)))
    }
}

pub struct DateDepKey;

impl DependencyKey for DateDepKey {
    type Value = DateDep;

    fn live() -> Self::Value {
        DateDep(Arc::new(DateServiceLive))
    }

    fn try_test() -> Result<Self::Value, DependencyError> {
        Ok(DateDep(Arc::new(DateServiceMock)))
    }
}

pub struct WebDepKey;

impl DependencyKey for WebDepKey {
    type Value = WebDep;

    fn live() -> Self::Value {
        WebDep(Arc::new(CheckoutWebSocketLive))
    }

    fn try_test() -> Result<Self::Value, DependencyError> {
        Ok(WebDep(Arc::new(CheckoutWebSocketMock)))
    }
}

pub fn shop_environment_live() -> Environment {
    Environment::from_values(
        DependencyValues::live()
            .with(HttpDepKey::live())
            .with(DateDepKey::live())
            .with(WebDepKey::live()),
    )
}

pub fn shop_environment_mock() -> Environment {
    Environment::from_values(
        DependencyValues::live()
            .with(HttpDepKey::test())
            .with(DateDepKey::test())
            .with(WebDepKey::test()),
    )
}
