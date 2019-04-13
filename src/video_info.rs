use futures::{future::Future, stream::Stream};
use serde::{Serialize, Deserialize};

use crate::errors::*;

const YOUTUBE_VIDEO_INFO_URL: &str = "https://www.youtube.com/get_video_info";

pub fn get_video_info(id: &str) -> impl Future<Item=VideoInfo, Error=Error> {
    let info_url = format!("{}?video_id={}", YOUTUBE_VIDEO_INFO_URL, id);

    crate::hyper_https::fetch_content(info_url.parse().unwrap()).map(|content| {
        dump_to_file("dump2.json", &serde_json::to_string_pretty(&serde_urlencoded::from_str::<serde_json::Value>(&content).unwrap()).unwrap());
        let v: VideoInfo = serde_urlencoded::from_str(&content).unwrap();
        dump_to_file("dump.json", &serde_json::to_string_pretty(&v).unwrap());
        v
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

#[derive(Serialize, Deserialize, Debug)]
pub struct VideoInfo {
    video_id: String,
    title: String,
    author: String,
    length_seconds: u64,
    thumbnail_url: String,
    #[serde(deserialize_with = "from_str")]
    player_response: PlayerResponse,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PlayerResponse {
    #[serde(rename = "streamingData")]
    streaming_data: StreamingData
}

#[derive(Serialize, Deserialize, Debug)]
pub struct StreamingData {
    #[serde(rename = "adaptiveFormats")]
    adaptive_formats: Vec<AdaptiveFormat>
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AdaptiveFormat {
    itag: u16,
    #[serde(rename = "mimeType")]
    mime_type: String,
    quality: String,
    url: String
}

fn dump_to_file(file_name: &str, text: &str) {
    use std::io::Write;
    let mut file = std::fs::OpenOptions::new()
            .write(true)
            .create(true)
            .open(file_name).unwrap();
    file.write(text.as_bytes()).unwrap();
}

fn from_str<'de, T, D>(deserializer: D) -> std::result::Result<T, D::Error>
    where T: std::str::FromStr,
          T::Err: std::fmt::Display,
          D: serde::de::Deserializer<'de>
{
    let s = String::deserialize(deserializer)?;
    T::from_str(&s).map_err(serde::de::Error::custom)
}

impl std::str::FromStr for PlayerResponse {
    type Err = serde_json::Error;
    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        serde_json::from_str(s)
    }
}