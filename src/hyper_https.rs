use std::io;
use std::sync::Arc;

use futures::{future::{err, Future}, stream::Stream};
use hyper::{Client, client::{connect::{Connect, Connected, Destination}, HttpConnector}};
use tokio::net::TcpStream;
use tokio_tls::{TlsConnector, TlsStream};

pub fn get_client() -> Client<HttpsConnector> {
    let tls_cx = native_tls::TlsConnector::builder().build().unwrap();
    let mut connector = HttpsConnector {
        tls: Arc::new(tls_cx.into()),
        http: HttpConnector::new(2),
    };
    connector.http.enforce_http(false);
    Client::builder().build(connector)
}

pub fn fetch_content(url: hyper::Uri) -> impl Future<Item=String, Error=hyper::error::Error> {
    let client = get_client();

    client.get(url)
        .and_then(|res| {
            res.into_body().concat2()
        })
        .map(|body| {
            String::from_utf8_lossy(&body).into_owned()
        })
}

pub struct HttpsConnector {
    tls: Arc<TlsConnector>,
    http: HttpConnector,
}

impl Connect for HttpsConnector {
    type Transport = TlsStream<TcpStream>;
    type Error = io::Error;
    type Future = Box<Future<Item=(Self::Transport, Connected), Error=Self::Error> + Send>;

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
