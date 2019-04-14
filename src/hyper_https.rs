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
    open_stream(url)
        .and_then(|res| {
            res.into_body().concat2()
        })
        .map(|body| {
            String::from_utf8_lossy(&body).into_owned()
        })
}

pub fn download_to_file(file_name: &str, url: hyper::Uri) -> impl Future<Item=(), Error=crate::errors::Error> {
    let response_future = open_stream(url).map_err(|e| e.into());

    let create_file_future =
        tokio::fs::File::create(file_name.to_owned()).map_err(|e| e.into());

    response_future
        .join(create_file_future)
        .and_then(move |(res, file)| {
            let len = res.headers().get(hyper::header::CONTENT_LENGTH).unwrap().to_str().unwrap().parse::<u64>().unwrap(); //RUST IN A NUTSHELL
            
            let pb = indicatif::ProgressBar::new(len);
            pb.enable_steady_tick(500);
            pb.set_style(indicatif::ProgressStyle::default_bar()
                .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {bytes}/{total_bytes} ({eta})")
                .progress_chars("#>-"));

            res.into_body()
                .map_err(|e| e.into())
                .fold(file, move |file, chunk| {
                    pb.inc(chunk.len() as u64);

                    tokio_io::io::write_all(file, chunk)
                        .map(|(f, _c)| f)
                        .map_err(|e| crate::errors::Error::from(e)) //compiler explodes if I use e.into() what the heck?!
                })
                .map(drop)
        })
}

pub async fn async_download_to_file(file_name: &str, url: hyper::Uri) -> Result<(), crate::errors::Error> {
    use tokio::prelude::*;

    let res = await!(open_stream(url))?;

    if res.status() != 200 {
        bail!("Failed to connect, status: {}", res.status());
    } else {
        println!("Connected! Status: {}", res.status());
    }

    let mut file = await!(tokio::fs::File::create(file_name.to_owned()))?;
    
    let len = res.headers().get(hyper::header::CONTENT_LENGTH).unwrap().to_str().unwrap().parse::<u64>().unwrap();

    let pb = indicatif::ProgressBar::new(len);
            pb.enable_steady_tick(100);
            pb.set_style(indicatif::ProgressStyle::default_bar()
                .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {bytes}/{total_bytes} ({eta})")
                .progress_chars("#>-"));

    let mut body = res.into_body();

    while let Some(chunk) = await!(body.next()) {
        let chunk = chunk?;
        pb.inc(chunk.len() as u64);
        await!(tokio_io::io::write_all(&mut file, chunk))?;
    }

    Ok(())
}

pub fn open_stream(url: hyper::Uri) -> hyper::client::ResponseFuture {
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
