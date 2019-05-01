use std::io;
use std::sync::Arc;

use futures::{future::{err, Future}, stream::Stream};
use hyper::{Client, client::{connect::{Connect, Connected, Destination}, HttpConnector}};
use tokio::net::TcpStream;
use tokio_tls::{TlsConnector, TlsStream};
use crate::error::Result;

/// Returns a `String` from the data read from the response.
/// Errors if it fails to connect.
pub(crate) async fn fetch_content(url: hyper::Uri) -> Result<String> {
    let res = await!(get_client().get(url))?;

    if res.status() != 200 {
        bail!("Failed to connect, status: {}", res.status());
    }

    let s = await!(res.into_body().concat2())?;
    Ok(String::from_utf8_lossy(&s).into_owned())
}

/// Return an HTTPS-only client for Hyper.
pub(crate) fn get_client() -> Client<HttpsConnector> {
    let tls_cx = native_tls::TlsConnector::builder().build().unwrap();
    let mut connector = HttpsConnector {
        tls: Arc::new(tls_cx.into()),
        http: HttpConnector::new(2),
    };
    connector.http.enforce_http(false);
    Client::builder().build(connector)
}

/// HTTPS connector for Hyper.
pub(crate) struct HttpsConnector {
    tls: Arc<TlsConnector>,
    http: HttpConnector,
}

impl Connect for HttpsConnector {
    type Transport = TlsStream<TcpStream>;
    type Error = io::Error;
    type Future = Box<Future<Item = (Self::Transport, Connected), Error = Self::Error> + Send>;

    fn connect(&self, dst: Destination) -> Self::Future {
        if dst.scheme() != "https" {
            return Box::new(err(io::Error::new(
                io::ErrorKind::Other,
                "only works with https",
            )));
        }

        let host = format!(
            "{}{}",
            dst.host(),
            dst.port().map(|p| format!(":{}", p)).unwrap_or("".into())
        );

        let tls_cx = self.tls.clone();
        Box::new(self.http.connect(dst).and_then(move |(tcp, connected)| {
            tls_cx
                .connect(&host, tcp)
                .map(|s| (s, connected))
                .map_err(|e| io::Error::new(io::ErrorKind::Other, e))
        }))
    }
}
