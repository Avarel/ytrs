use std::io;
use std::sync::Arc;

use futures::{future::{err, Future}, stream::Stream};
use hyper::{Client, client::{connect::{Connect, Connected, Destination}, HttpConnector}};
use tokio::net::TcpStream;
use tokio_tls::{TlsConnector, TlsStream};
use tokio::prelude::*;

pub fn get_client() -> Client<HttpsConnector> {
    let tls_cx = native_tls::TlsConnector::builder().build().unwrap();
    let mut connector = HttpsConnector {
        tls: Arc::new(tls_cx.into()),
        http: HttpConnector::new(2),
    };
    connector.http.enforce_http(false);
    Client::builder().build(connector)
}

pub async fn fetch_content(url: hyper::Uri) -> Result<String, hyper::error::Error> {
    let res = await!(get_response_async(url))?;
    let s = await!(res.into_body().concat2())?;
    Ok(String::from_utf8_lossy(&s).into_owned())
}

pub async fn async_download_to_file(file_name: &str, url: hyper::Uri) -> Result<(), crate::errors::Error> {
    let (mut stream, len) = await!(open_download_stream(url))?;

    let pb = indicatif::ProgressBar::new(len);
            pb.enable_steady_tick(100);
            pb.set_style(indicatif::ProgressStyle::default_bar()
                .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {bytes}/{total_bytes} ({eta})")
                .progress_chars("#>-"));

    let mut file = await!(tokio::fs::File::create(file_name.to_owned()))?;

    while let Some(chunk) = await!(stream.next()) {
        let chunk = chunk?;
        pb.inc(chunk.len() as u64);
        await!(tokio_io::io::write_all(&mut file, chunk))?;
    }

    Ok(())
}

pub async fn open_download_stream(url: hyper::Uri) -> Result<(hyper::Body, u64), crate::errors::Error> {
    let res = await!(get_response_async(url))?;

    if res.status() != 200 {
        bail!("Failed to connect, status: {}", res.status());
    } else {
        println!("Connected! Status: {}", res.status());
    }

    let len = res.headers().get(hyper::header::CONTENT_LENGTH).unwrap().to_str().unwrap().parse::<u64>().unwrap();

    Ok((res.into_body(), len))
}

fn get_response_async(url: hyper::Uri) -> hyper::client::ResponseFuture {
    let client = get_client();
    client.get(url)
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
