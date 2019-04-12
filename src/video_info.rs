use futures::{future::Future, stream::Stream};
use serde::Deserialize;

use crate::errors::*;

const YOUTUBE_VIDEO_INFO_URL: &str = "https://www.youtube.com/get_video_info";

pub fn get_video_info(id: &str) -> impl Future<Item=VideoInfo, Error=Error> {
    let info_url = format!("{}?video_id={}", YOUTUBE_VIDEO_INFO_URL, id);

    crate::hyper_https::fetch_content(info_url.parse().unwrap()).map(|content| {
        let v: VideoInfo = serde_urlencoded::from_str(&content).unwrap();
        dbg!(v)
    }).map_err(|e| e.into())
}

pub fn get_id_from_string(s: &str) -> Result<String> {
    let start = if s.contains("youtube.com/") {
        s.find("?v=").ok_or("?v= not found")? + 3
    } else if let Some(index) = s.find("youtu.be/") {
        index + 8
    } else {
        bail!("Invalid schema/host")
    };

    let end = s[start..].find('?').unwrap_or(s.len());
    return Ok(s[start..end].to_owned())
}

#[derive(Deserialize, Debug)]
pub struct VideoInfo {
    title: String,
    author: String,
    length_seconds: u64,
}