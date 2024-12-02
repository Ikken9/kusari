use crate::{Request, Response};
use bytes::Bytes;
use http::{HeaderMap, HeaderName, StatusCode};
use rustls::{ClientConfig, RootCertStore};
use std::error::Error;
use std::str::FromStr;
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::sync::Mutex;
use tokio_rustls::client::TlsStream;
use tokio_rustls::TlsConnector;
use url::Url;
use http::header::CONTENT_LENGTH;
use http::HeaderValue;


#[derive(Clone)]
pub struct Client {
    tls_config: Arc<ClientConfig>,
    tls_stream: Option<Arc<Mutex<TlsStream<TcpStream>>>>,
}

pub trait HeaderValueExt {
    fn to_string(&self) -> String;
}

impl HeaderValueExt for HeaderValue {
    fn to_string(&self) -> String {
        self.to_str().unwrap_or_default().to_string()
    }
}

impl Client {
    pub fn new() -> Self {
        let root_cert_store = RootCertStore {
            roots: webpki_roots::TLS_SERVER_ROOTS.into()
        };

        let mut tls_config = ClientConfig::builder()
            .with_root_certificates(root_cert_store)
            .with_no_client_auth();

        tls_config.key_log = Arc::new(rustls::KeyLogFile::new());

        Client {
            tls_config: Arc::new(tls_config),
            tls_stream: None,
        }
    }

    pub async fn connect(&mut self, url: Url) {
        let host = url.host_str().ok_or("Invalid host").unwrap().to_string();
        let port = match url.port() {
            Some(port) => port,
            None => if url.scheme() == "https" { 443 } else { 80 },
        };

        let addr = format!("{}:{}", host, port);

        let stream = TcpStream::connect(addr).await.unwrap();
        let connector = TlsConnector::from(self.tls_config.clone());
        let domain = rustls_pki_types::ServerName::try_from(host).unwrap();
        self.tls_stream = Some(Arc::new(Mutex::new(connector.connect(domain, stream).await.unwrap())));
    }

    pub async fn send_request(&mut self, request: Request, url: Url) -> Result<Response, Box<dyn Error + Send + Sync>> {
        let mut request_str = format!("{} {} HTTP/1.1\r\n", request.method, url.path());
        request_str += &format!("Host: {}\r\n", url.host_str().unwrap_or(""));

        for (key, value) in &request.headers {
            request_str += &format!("{}: {:?}\r\n", key.as_str(), value);
        }

        if !request.body.is_empty() {
            request_str += &format!("Content-Length: {}\r\n", request.body.len());
        }

        request_str += "\r\n";

        self.clone().tls_stream.unwrap().lock().await.write_all(request_str.as_bytes()).await?;
        if !request.body.is_empty() {
            self.clone().tls_stream.unwrap().lock().await.write_all(&request.body).await?;
        }

        let response = {
            let mut stream = self.tls_stream.as_ref().unwrap().lock().await;
            Self::read_response(&mut *stream).await?
        };

        Ok(response)
    }

    pub async fn read_response(stream: &mut TlsStream<TcpStream>) -> Result<Response, Box<dyn Error + Send + Sync>> {
        let mut buffer = Vec::new();
        let mut headers = [0; 8192]; // 8KB for headers

        loop {
            let n = stream.read(&mut headers).await?;
            if n == 0 {
                break;
            }
            buffer.extend_from_slice(&headers[..n]);
            if buffer.windows(4).any(|w| w == b"\r\n\r\n") {
                break;
            }
        }

        let (header_part, body_start) = {
            let header_end = buffer.windows(4).position(|w| w == b"\r\n\r\n").ok_or("Invalid response")? + 4;
            (&buffer[..header_end], &buffer[header_end..])
        };

        let response_str = String::from_utf8_lossy(header_part);
        let mut lines = response_str.lines();

        let status_line = lines.next().ok_or("Missing status line")?;
        let mut status_parts = status_line.splitn(3, ' ');
        let _http_version = status_parts.next().ok_or("Missing HTTP version")?;
        let status_code = status_parts.next().ok_or("Missing status code")?.parse::<u16>()?;
        let status = StatusCode::from_u16(status_code)?;
        let _reason_phrase = status_parts.next().unwrap_or("");

        let mut headers_map = HeaderMap::new();
        for line in lines {
            if line.is_empty() {
                break;
            }
            if let Some((key, value)) = line.split_once(": ") {
                let header_name = HeaderName::from_str(key)?;
                let header_value = HeaderValue::from_str(value)?;
                headers_map.insert(header_name, header_value);
            }
        }

        let content_length = if let Some(value) = headers_map.get(CONTENT_LENGTH) {
            Some(value.to_string().parse::<usize>()?)
        } else {
            None
        };

        let mut body = body_start.to_vec();

        if let Some(content_length) = content_length {
            let mut remaining = content_length - body.len();
            while remaining > 0 {
                let mut buf = vec![0; remaining];
                let n = stream.read(&mut buf).await?;
                if n == 0 {
                    break;
                }
                body.extend_from_slice(&buf[..n]);
                remaining -= n;
            }
        }

        Ok(Response {
            status_code,
            status,
            headers: headers_map,
            body: Bytes::from(body).to_vec(),
            reason_phrase: "".to_string(),
        })
    }
}