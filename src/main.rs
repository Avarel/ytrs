use hyper::rt::{Future, Stream};

#[macro_use]
extern crate error_chain;

mod hyper_https;
mod video_info;
mod errors;


fn main() {
    let url_str = match std::env::args().nth(1) {
        Some(url) => url,
        None => {
            println!("Usage: client <url>");
            return;
        }
    };

    let url = url_str.parse::<hyper::Uri>().unwrap();

    if let Some(id) = video_info::get_id_from_string(&url.to_string()).ok() {
        println!("Discovered youtube id {:?}", id);
        return;
    }

    let mut runtime = tokio::runtime::Runtime::new().expect("Unable to create a runtime");
    let s = runtime.block_on(fetch_url(url)).unwrap();
    println!("{}", s);
}

fn fetch_url(url: hyper::Uri) -> impl Future<Item=String, Error=hyper::error::Error> {
    let client = hyper_https::get_client();

    client.get(url)
        .and_then(|res| {
            res.into_body().concat2()
        })
        .map(|body| {
            String::from_utf8_lossy(&body).into_owned()
        })
}