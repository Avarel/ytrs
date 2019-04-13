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
    let download = std::env::args().nth(1).map(|s| s == "download").unwrap_or(false);

    if download {
        let link = "XXXX";
        let url = link.parse::<hyper::Uri>().unwrap();
        let mut runtime = tokio::runtime::Runtime::new().expect("Unable to create a runtime");
        let result = runtime.block_on(hyper_https::download_to_file("what.webm", url));
        match result {
            Ok(()) => println!("Finished!"),
            Err(msg) => println!("Error: {}", msg),
        }
        return;
    }
    
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

