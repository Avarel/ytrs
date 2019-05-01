#![feature(await_macro, async_await, inner_deref)]

// This pulls in the `tokio-async-await` crate. While Rust 2018 doesn't require
// `extern crate`, we need to pull in the macros.
#[macro_use]
extern crate tokio;

#[macro_use]
extern crate error_chain;

#[macro_use]
pub mod error;
// #[cfg(feature = "download")]
pub mod download;
pub mod hyper_https;
pub mod video_info;

use video_info::{VideoInfo, Format, FormatDetails};
use tokio::prelude::*;

fn main() {
    tokio::run_async(async {
        if let Err(error) = await!(run_cli()) {
            println!("Error occured: {:?}", error);
        }
    });
}

async fn run_cli() -> error::Result<()> {
    let args = std::env::args().collect::<Vec<_>>();
    match args.get(1).map(|s| s.as_str()) {
        Some("download") | Some("get") => {
            let url = args.get(2).ok_or("Expected a url")?;
            let id = video_info::get_id_from_string(url)?;
            
            let info = await!(video_info::get_video_info(&id)).unwrap();
            print_info(&info);
            println!("{:>5}: {:5} | {:<10} | {:<15} | {:<10} | ", "index", "itag", "bitrate", "codec", "extension");
            for (i, format) in info.formats.iter().enumerate() {
                print_format(i, format);
            }
            println!("Type the index that you want to download to the current directory.");

            let std_in = std::io::stdin();
            let mut line = String::new();
            std_in.read_line(&mut line)?;
            let index = line.trim().parse::<usize>().map_err(|_| "Not a number")?;

            if let Some(format) = info.formats.get(index) {
                let (mut stream, len) = await!(format.download().open_stream())?;

                let name = format!("{}.{}", "testing", format.extension);
        
                let file = std::fs::OpenOptions::new()
                    .write(true)
                    .create(true)
                    .open(name)?;

                file.set_len(0)?;

                let mut file = tokio::fs::File::from_std(file);

                let pb = indicatif::ProgressBar::new(len);
                pb.enable_steady_tick(100);
                pb.set_style(indicatif::ProgressStyle::default_bar()
                    .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {bytes}/{total_bytes} ({eta})")
                    .progress_chars("#>-"));

                while let Some(chunk) = await!(stream.next()) {
                    let chunk = chunk?;
                    pb.inc(chunk.len() as u64);
                    await!(tokio::io::write_all(&mut file, chunk))?;
                }

                pb.finish_with_message("Finished downloading!");

                return Ok(())
            } else {
                bail!(format!("Not an index within 0..{}", info.formats.len()))
            }
        }
        Some("info") | Some("i") => {
            let url = args.get(2).ok_or("Expected a url")?;
            let id = video_info::get_id_from_string(url)?;
            
            let info = await!(video_info::get_video_info(&id)).unwrap();
            print_info(&info);
        }
        _ => {
            println!("Need more arguments:");
            println!("\t(i, info) (url) - Show information about the video.");
            println!("\t(get, download) (url) - Download a video.")
        }
    }

    Ok(())
}

fn print_info(info: &VideoInfo) {
    println!("[video.information]");
    println!("title = {}", info.details.title);
    println!("id = {}", info.details.video_id);
    println!("author = {}", info.details.author);
    println!("duration = {} seconds", info.length_seconds);
    println!("views = {}", info.details.view_count);
}

fn print_format(index: usize, format: &Format) {
    print!("{:>5}: {:>5} | {:<10} | {:<15} | {:<10} | ", index, format.itag, format.bitrate, format.codec, format.extension);
    match &format.details {
        FormatDetails::Audio { audio_channels, audio_sample_rate } => {
            println!("audio {:<10} | {:<10}", format!("{} Hz", audio_sample_rate), format!("{} channels", audio_channels));
        }
        FormatDetails::Video { fps, size, quality_label } => {
            println!("video {:<10} | {:<10} | {:<10}", quality_label, format!("{} fps", fps), size);
        }
    };
}