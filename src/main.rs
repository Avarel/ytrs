#[macro_use]
extern crate error_chain;

#[macro_use]
mod errors;

mod hyper_https;

mod video_info;

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
        let mut runtime = tokio::runtime::Runtime::new().expect("Unable to create a runtime");
        runtime.block_on(video_info::get_video_info(&id)).unwrap();
        return;
    }

    let mut runtime = tokio::runtime::Runtime::new().expect("Unable to create a runtime");
    let s = runtime.block_on(hyper_https::fetch_content(url)).unwrap();
    println!("{}", s);
}