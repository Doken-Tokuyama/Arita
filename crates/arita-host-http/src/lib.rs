//! ADR-245 — curated HTTP host border (types surface). Opaque server = i64.
#![forbid(unsafe_code)]

use std::collections::HashMap;
use std::convert::Infallible;
use std::future::Future;
use std::net::SocketAddr;
use std::sync::atomic::{AtomicBool, AtomicI64, Ordering};
use std::sync::{Arc, Mutex};

use bytes::Bytes;
use http_body_util::{BodyExt, Full, Limited};
use hyper::body::Incoming;
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper::{Request, Response, StatusCode};
use hyper_util::rt::TokioIo;
use tokio::net::TcpListener;
use tokio::sync::RwLock;

pub const MAX_BODY_BYTES: usize = 1024 * 1024; // default; prefer ServicePolicy
pub const MAX_HEADERS_BYTES: usize = 64 * 1024;

/// Surface `IoError` (stable; never bare String on ARITA Result).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IoError {
    pub code: i64,
    pub message: String,
}

impl IoError {
    pub fn new(code: i64, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
        }
    }
    pub fn msg(message: impl Into<String>) -> Self {
        Self::new(1, message)
    }
}

impl std::fmt::Display for IoError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} ({})", self.message, self.code)
    }
}

/// ADR-246 — enforceable service caps (defaults match ADR).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ServicePolicy {
    pub max_body_bytes: i64,
    pub max_header_bytes: i64,
    pub max_header_count: i64,
    pub idle_timeout_ms: i64,
}

impl ServicePolicy {
    pub fn default_policy() -> Self {
        Self {
            max_body_bytes: 1_048_576,
            max_header_bytes: 65_536,
            max_header_count: 100,
            idle_timeout_ms: 30_000,
        }
    }

    /// v0: every field must be > 0.
    pub fn validate(&self) -> Result<(), IoError> {
        for (name, v) in [
            ("max_body_bytes", self.max_body_bytes),
            ("max_header_bytes", self.max_header_bytes),
            ("max_header_count", self.max_header_count),
            ("idle_timeout_ms", self.idle_timeout_ms),
        ] {
            if v <= 0 {
                return Err(IoError::msg(format!("invalid policy value: {name}={v}")));
            }
        }
        Ok(())
    }
}

impl Default for ServicePolicy {
    fn default() -> Self {
        Self::default_policy()
    }
}

static NEXT_ID: AtomicI64 = AtomicI64::new(1);
static SERVERS: Mutex<Option<HashMap<i64, Arc<ServerInner>>>> = Mutex::new(None);

fn with_map<R>(f: impl FnOnce(&mut HashMap<i64, Arc<ServerInner>>) -> R) -> R {
    let mut g = SERVERS.lock().unwrap_or_else(|e| e.into_inner());
    if g.is_none() {
        *g = Some(HashMap::new());
    }
    f(g.as_mut().unwrap())
}

struct ServerInner {
    port: AtomicI64,
    listener: RwLock<Option<TcpListener>>,
    shutdown: AtomicBool,
    policy: Mutex<ServicePolicy>,
}

#[derive(Clone, Debug)]
pub struct HttpRequest {
    pub method: String,
    pub path: String,
    pub headers: Vec<(String, String)>,
    pub body_text: String,
}

impl HttpRequest {
    pub fn method(&self) -> String {
        self.method.clone()
    }
    pub fn path(&self) -> String {
        self.path.clone()
    }
    pub fn header(&self, name: String) -> Option<String> {
        let want = name.to_ascii_lowercase();
        self.headers
            .iter()
            .find(|(k, _)| k.to_ascii_lowercase() == want)
            .map(|(_, v)| v.clone())
    }
    pub fn body_text(&self) -> Result<String, IoError> {
        Ok(self.body_text.clone())
    }
}

#[derive(Clone, Debug)]
pub struct HttpResponse {
    pub status: i64,
    pub headers: Vec<(String, String)>,
    pub body: String,
}

impl HttpResponse {
    pub fn ok_text(body: String) -> Self {
        Self {
            status: 200,
            headers: vec![("content-type".into(), "text/plain; charset=utf-8".into())],
            body,
        }
    }
    pub fn status(code: i64, body: String) -> Self {
        Self {
            status: code,
            headers: vec![("content-type".into(), "text/plain; charset=utf-8".into())],
            body,
        }
    }
    pub fn set_header(mut self, name: String, value: String) -> Self {
        self.headers.push((name, value));
        self
    }

    /// ADR-247: instance `resp.status()` — Rust field read via emit (name clash with associated `status`).
    pub fn status_code(&self) -> i64 {
        self.status
    }

    /// ADR-247: `resp.body_text() -> Result[Text, IoError]`
    pub fn body_text(&self) -> Result<String, IoError> {
        Ok(self.body.clone())
    }
}

/// `HttpServer.bind(addr, port) -> Result<HttpServer, IoError>` (opaque i64).
pub async fn bind(addr: String, port: i64) -> Result<i64, IoError> {
    if !(0..=65535).contains(&port) {
        return Err(IoError::msg("bind: port out of range"));
    }
    let ip: std::net::IpAddr = addr
        .parse()
        .map_err(|_| IoError::msg("bind: invalid addr"))?;
    let sock = SocketAddr::from((ip, port as u16));
    let listener = TcpListener::bind(sock)
        .await
        .map_err(|e| IoError::msg(format!("bind: {e}")))?;
    let bound = listener
        .local_addr()
        .map_err(|e| IoError::msg(format!("bind: {e}")))?;
    let id = NEXT_ID.fetch_add(1, Ordering::SeqCst);
    let inner = Arc::new(ServerInner {
        port: AtomicI64::new(bound.port() as i64),
        listener: RwLock::new(Some(listener)),
        shutdown: AtomicBool::new(false),
        policy: Mutex::new(ServicePolicy::default_policy()),
    });
    with_map(|m| {
        m.insert(id, inner);
    });
    Ok(id)
}

pub fn port(server: i64) -> i64 {
    with_map(|m| {
        m.get(&server)
            .map(|s| s.port.load(Ordering::SeqCst))
            .unwrap_or(0)
    })
}

pub fn shutdown(server: i64) {
    with_map(|m| {
        if let Some(s) = m.get(&server) {
            s.shutdown.store(true, Ordering::SeqCst);
        }
    });
}

/// ADR-246: `server.set_policy(p) -> Result<(), IoError>` (opaque id unchanged).
pub fn set_policy(server: i64, policy: ServicePolicy) -> Result<(), IoError> {
    policy.validate()?;
    with_map(|m| {
        let s = m
            .get(&server)
            .ok_or_else(|| IoError::msg("set_policy: bad server"))?;
        *s.policy.lock().unwrap_or_else(|e| e.into_inner()) = policy;
        Ok(())
    })
}

/// ADR-246: `server.policy() -> ServicePolicy`
pub fn get_policy(server: i64) -> ServicePolicy {
    with_map(|m| {
        m.get(&server)
            .map(|s| s.policy.lock().unwrap_or_else(|e| e.into_inner()).clone())
            .unwrap_or_else(ServicePolicy::default_policy)
    })
}

/// Catch-all serve: one async handler per request.
pub async fn serve<F, Fut>(server: i64, handler: F) -> Result<(), IoError>
where
    F: Fn(HttpRequest) -> Fut + Send + Sync + 'static + Clone,
    Fut: Future<Output = Result<HttpResponse, IoError>> + Send + 'static,
{
    let inner =
        with_map(|m| m.get(&server).cloned()).ok_or_else(|| IoError::msg("serve: bad server"))?;
    let listener = inner
        .listener
        .write()
        .await
        .take()
        .ok_or_else(|| IoError::msg("serve: already serving or not listening"))?;

    loop {
        if inner.shutdown.load(Ordering::SeqCst) {
            break;
        }
        let accept =
            tokio::time::timeout(std::time::Duration::from_millis(200), listener.accept()).await;
        let (stream, _) = match accept {
            Ok(Ok(x)) => x,
            Ok(Err(e)) => return Err(IoError::msg(format!("serve: {e}"))),
            Err(_) => continue,
        };
        let io = TokioIo::new(stream);
        let h = handler.clone();
        let caps = {
            let g = inner.policy.lock().unwrap_or_else(|e| e.into_inner());
            (
                g.max_body_bytes.max(1) as usize,
                g.max_header_bytes.max(1) as usize,
                g.max_header_count.max(1) as usize,
            )
        };
        // TODO(ADR-244/246): idle_timeout_ms not yet enforced on accept/read.
        tokio::spawn(async move {
            let svc = service_fn(move |req| {
                let h = h.clone();
                let caps = caps;
                async move { Ok::<_, Infallible>(dispatch(h, caps, req).await) }
            });
            let _ = http1::Builder::new().serve_connection(io, svc).await;
        });
    }
    Ok(())
}

async fn dispatch<F, Fut>(
    handler: F,
    caps: (usize, usize, usize),
    req: Request<Incoming>,
) -> Response<Full<Bytes>>
where
    F: Fn(HttpRequest) -> Fut,
    Fut: Future<Output = Result<HttpResponse, IoError>>,
{
    let (max_body, max_header_bytes, max_header_count) = caps;
    let method = req.method().clone();
    let path = req.uri().path().to_string();
    let mut header_bytes = 0usize;
    let mut headers = Vec::new();
    for (k, v) in req.headers().iter() {
        if headers.len() >= max_header_count {
            return Response::builder()
                .status(StatusCode::REQUEST_HEADER_FIELDS_TOO_LARGE)
                .body(Full::new(Bytes::from_static(b"too many headers")))
                .unwrap_or_else(|_| Response::new(Full::new(Bytes::new())));
        }
        header_bytes = header_bytes.saturating_add(k.as_str().len() + v.len());
        if header_bytes > max_header_bytes {
            return Response::builder()
                .status(StatusCode::REQUEST_HEADER_FIELDS_TOO_LARGE)
                .body(Full::new(Bytes::from_static(b"headers too large")))
                .unwrap_or_else(|_| Response::new(Full::new(Bytes::new())));
        }
        headers.push((
            k.as_str().to_string(),
            String::from_utf8_lossy(v.as_bytes()).into_owned(),
        ));
    }
    let collected = Limited::new(req.into_body(), max_body).collect().await;
    let body_text = match collected {
        Ok(c) => String::from_utf8_lossy(&c.to_bytes()).into_owned(),
        Err(_) => {
            return Response::builder()
                .status(StatusCode::PAYLOAD_TOO_LARGE)
                .body(Full::new(Bytes::from_static(b"body too large")))
                .unwrap_or_else(|_| Response::new(Full::new(Bytes::new())));
        }
    };
    let snap = HttpRequest {
        method: method.as_str().to_string(),
        path,
        headers,
        body_text,
    };
    match handler(snap).await {
        Ok(out) => {
            let status = StatusCode::from_u16(out.status.clamp(100, 599) as u16)
                .unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);
            let mut b = Response::builder().status(status);
            for (k, v) in &out.headers {
                b = b.header(k.as_str(), v.as_str());
            }
            b.body(Full::new(Bytes::from(out.body)))
                .unwrap_or_else(|_| Response::new(Full::new(Bytes::new())))
        }
        Err(e) => Response::builder()
            .status(StatusCode::INTERNAL_SERVER_ERROR)
            .body(Full::new(Bytes::from(e.message)))
            .unwrap_or_else(|_| Response::new(Full::new(Bytes::new()))),
    }
}

// --- ADR-247 curated HTTP client (surface: HttpClient.*; no reqwest) ---

struct ClientUrl {
    host: String,
    port: u16,
    path: String,
}

fn parse_http_url(url: &str) -> Result<ClientUrl, IoError> {
    let rest = url
        .strip_prefix("http://")
        .ok_or_else(|| IoError::msg("client: only http:// URLs in v0"))?;
    let (hostport, path) = match rest.split_once('/') {
        Some((hp, p)) => (hp, format!("/{p}")),
        None => (rest, "/".to_string()),
    };
    let (host, port) = if let Some((h, p)) = hostport.split_once(':') {
        let port: u16 = p.parse().map_err(|_| IoError::msg("client: bad port"))?;
        (h.to_string(), port)
    } else {
        (hostport.to_string(), 80u16)
    };
    if host.is_empty() {
        return Err(IoError::msg("client: empty host"));
    }
    Ok(ClientUrl { host, port, path })
}

async fn http_exchange(
    method: &str,
    url: String,
    body: Option<String>,
) -> Result<HttpResponse, IoError> {
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::net::TcpStream;

    let u = parse_http_url(&url)?;
    let addr = format!("{}:{}", u.host, u.port);
    let mut stream = TcpStream::connect(&addr)
        .await
        .map_err(|e| IoError::msg(format!("client connect: {e}")))?;

    let body_s = body.unwrap_or_default();
    let mut req = format!(
        "{method} {path} HTTP/1.1\r\nHost: {host}:{port}\r\nConnection: close\r\nUser-Agent: arita-host-http/0.2\r\n",
        method = method,
        path = u.path,
        host = u.host,
        port = u.port,
    );
    if method == "POST" {
        req.push_str("Content-Type: text/plain; charset=utf-8\r\n");
        req.push_str(&format!("Content-Length: {}\r\n", body_s.len()));
    }
    req.push_str("\r\n");
    req.push_str(&body_s);

    stream
        .write_all(req.as_bytes())
        .await
        .map_err(|e| IoError::msg(format!("client write: {e}")))?;

    let mut buf = Vec::new();
    stream
        .read_to_end(&mut buf)
        .await
        .map_err(|e| IoError::msg(format!("client read: {e}")))?;
    let raw = String::from_utf8_lossy(&buf);
    let (head, body_out) = raw.split_once("\r\n\r\n").unwrap_or((raw.as_ref(), ""));
    let status_line = head.lines().next().unwrap_or("");
    let code = status_line
        .split_whitespace()
        .nth(1)
        .and_then(|c| c.parse::<i64>().ok())
        .unwrap_or(0);
    if code == 0 {
        return Err(IoError::msg(format!(
            "client: bad status line {status_line:?}"
        )));
    }
    Ok(HttpResponse {
        status: code,
        headers: vec![("content-type".into(), "text/plain; charset=utf-8".into())],
        body: body_out.to_string(),
    })
}

/// ADR-247 `HttpClient.get(url)`
/// ADR-247 optional helper: `HttpClient.url(port, path)`
pub fn client_url(port: i64, path: String) -> String {
    let p = if path.starts_with('/') {
        path
    } else {
        format!("/{path}")
    };
    format!("http://127.0.0.1:{port}{p}")
}

pub async fn client_get(url: String) -> Result<HttpResponse, IoError> {
    http_exchange("GET", url, None).await
}

/// ADR-247 `HttpClient.post_text(url, body)`
pub async fn client_post_text(url: String, body: String) -> Result<HttpResponse, IoError> {
    http_exchange("POST", url, Some(body)).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn policy_defaults_and_set() {
        let d = ServicePolicy::default_policy();
        assert_eq!(d.max_body_bytes, 1_048_576);
        assert_eq!(d.max_header_bytes, 65_536);
        assert_eq!(d.max_header_count, 100);
        assert_eq!(d.idle_timeout_ms, 30_000);
        assert!(ServicePolicy {
            max_body_bytes: 0,
            ..d.clone()
        }
        .validate()
        .is_err());
        assert!(ServicePolicy {
            max_body_bytes: -1,
            ..d
        }
        .validate()
        .is_err());
    }

    #[tokio::test]
    async fn set_policy_roundtrip() {
        let s = bind("127.0.0.1".into(), 0).await.expect("bind");
        let p = get_policy(s);
        assert_eq!(p.max_body_bytes, 1_048_576);
        let mut custom = ServicePolicy::default_policy();
        custom.max_body_bytes = 16;
        set_policy(s, custom.clone()).expect("set");
        assert_eq!(get_policy(s).max_body_bytes, 16);
        assert!(set_policy(
            s,
            ServicePolicy {
                max_body_bytes: 0,
                ..custom
            }
        )
        .is_err());
        shutdown(s);
    }

    #[tokio::test]
    async fn client_get_health_roundtrip() {
        let s = bind("127.0.0.1".into(), 0).await.expect("bind");
        let p = port(s);
        let handle = tokio::spawn(async move {
            let _ = serve(s, |req| async move {
                if req.path() == "/health" {
                    Ok(HttpResponse::ok_text("ok".into()))
                } else {
                    Ok(HttpResponse::status(404, "no".into()))
                }
            })
            .await;
        });
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        let resp = client_get(format!("http://127.0.0.1:{p}/health"))
            .await
            .expect("get");
        assert_eq!(resp.status_code(), 200);
        assert_eq!(resp.body_text().unwrap(), "ok");
        shutdown(s);
        let _ = handle.await;
    }

    #[tokio::test]
    async fn bind_and_handler_health() {
        let s = bind("127.0.0.1".into(), 0).await.expect("bind");
        assert!(port(s) > 0);
        let p = port(s);
        let h = tokio::spawn(async move {
            serve(s, |req| async move {
                if req.path() == "/health" {
                    Ok(HttpResponse::ok_text("ok".into()))
                } else {
                    Ok(HttpResponse::status(404, "no".into()))
                }
            })
            .await
        });
        tokio::time::sleep(std::time::Duration::from_millis(80)).await;
        let url = format!("http://127.0.0.1:{p}/health");
        let out = tokio::process::Command::new("curl")
            .args(["-sS", "-m", "2", &url])
            .output()
            .await;
        shutdown(s);
        let _ = h.await;
        if let Ok(o) = out {
            assert_eq!(String::from_utf8_lossy(&o.stdout), "ok");
        }
    }
}
