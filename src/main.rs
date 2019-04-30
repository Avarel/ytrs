#![feature(await_macro, async_await)]

// This pulls in the `tokio-async-await` crate. While Rust 2018 doesn't require
// `extern crate`, we need to pull in the macros.
#[macro_use]
extern crate tokio;

#[macro_use]
extern crate error_chain;

#[macro_use]
mod error;

mod hyper_https;

mod video_info;

fn main() {
    match std::env::args().nth(1) {
        Some(ref arg) if arg == "download" => {
            tokio::run_async(download(read_from_file("url.txt")));
            println!("test");
            return;
        }
        Some(ref arg) if arg == "grab" => {
            let url = std::env::args().nth(2).expect("a link in the 2nd arg position");

            if let Some(id) = video_info::get_id_from_string(&url.to_string()).ok() {
                println!("Discovered youtube id {:?}", id);
                tokio::run_async(async move {
                    let v = await!(video_info::get_video_info(&id)).unwrap();
                    video_info::dump_to_file("dump.json", &serde_json::to_string_pretty(&v).unwrap());
                });

            } else {
                println!("id not discovered");
            }
            return;
        }
        Some(url) => {
            let url = url.parse::<hyper::Uri>().unwrap();

            tokio::run_async(async {
                let s = await!(hyper_https::fetch_content(url)).unwrap();
                println!("{}", s);
            })
        }
        None => {
            println!("Usage: download | grab <url> | <url>");
            return;
        }
    };
}

fn read_from_file(path: &str) -> String {
    std::fs::read_to_string(path).expect("Unable to read file")
}

async fn download(link: String) {
    let url = link.parse::<hyper::Uri>().unwrap();
    await!(hyper_https::async_download_to_file("what.webm", url)).unwrap();
}