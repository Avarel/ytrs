use futures::{future::Future, stream::Stream};
use serde::{Serialize, Deserialize};

use crate::errors::*;

const YOUTUBE_VIDEO_INFO_URL: &str = "https://www.youtube.com/get_video_info";

pub fn get_video_info(id: &str) -> impl Future<Item=VideoInfo, Error=Error> {
    let info_url = format!("{}?video_id={}", YOUTUBE_VIDEO_INFO_URL, id);

    crate::hyper_https::fetch_content(info_url.parse().unwrap()).map(|content| {
        dump_to_file("dump2.json", &serde_json::to_string_pretty(&serde_urlencoded::from_str::<serde_json::Value>(&content).unwrap()).unwrap());
        VideoInfo::from_json(serde_urlencoded::from_str(&content).unwrap()).unwrap()
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

#[derive(Serialize, Debug)]
pub struct VideoInfo {
    video_id: String,
    title: String,
    length_seconds: u64,
    formats: Vec<Format>,
    details: VideoDetails
}

use serde_json::Value;
use std::str::FromStr;
impl VideoInfo {
    fn from_json(json: Value) -> Result<Self> {
        let context_version = json["innertube_context_client_version"]
            .as_str()
            .ok_or("Context client version not detected")?;
        if context_version != "1.20190423" {
            return Err("API context version out of date".into())
        }

        let video_id = json["video_id"].as_str().unwrap().to_owned();
        let title = json["title"].as_str().unwrap().to_owned();
        let length_seconds = json["length_seconds"].as_str().and_then(|s| u64::from_str(s).ok()).unwrap();

        let formats = json["adaptive_fmts"]
            .as_str()
            .unwrap()
            .split(',')
            .map(|s| serde_urlencoded::from_str::<Value>(s))
            .collect::<std::result::Result<Vec<_>, _>>()
            .map_err(|_| "Parsing error")?
            .iter()
            .map(|j| Format::parse_json(j))
            .collect::<Vec<_>>();

        let player_response = json["player_response"].as_str().and_then(|s| serde_json::from_str::<Value>(s).ok()).unwrap();
        let details = VideoDetails::parse_json(&player_response["videoDetails"]);

        Ok(Self {
            video_id,
            title,
            length_seconds,
            formats,
            details
        })
    }
}

#[derive(Serialize, Debug)]
#[serde(tag = "type")]
pub enum FormatDetails {
    Video {
        fps: u16,
        size: String,
        quality_label: String
    },
    Audio {
        audio_channels: u8,
        audio_sample_rate: u32
    }
}

#[derive(Serialize, Debug)]
pub struct Format {
    itag: u32,
    bitrate: u32,
    url: String,
    extension: String,
    codec: String,
    details: FormatDetails
}

impl Format {
    fn parse_json(json: &Value) -> Format {
        let itag = json["itag"].as_str().and_then(|s| u32::from_str(s).ok()).unwrap();
        let bitrate = json["bitrate"].as_str().and_then(|s| u32::from_str(s).ok()).unwrap();
        let url = json["url"].as_str().unwrap().to_owned();
        let (format_type, extension, codec) = json["type"].as_str().map(Self::parse_mime).unwrap();
        let details = Self::parse_details(&format_type, json);

        Self {
            itag,
            bitrate,
            url,
            extension,
            codec,
            details,
        }
    }

    fn parse_details(format_type: &str, json: &Value) -> FormatDetails {
        match format_type {
            "video" => {
                let fps = json["fps"].as_str().and_then(|s| u16::from_str(s).ok()).unwrap();
                let size = json["size"].as_str().unwrap().to_owned();
                let quality_label = json["quality_label"].as_str().unwrap().to_owned();
                FormatDetails::Video {
                    fps, size, quality_label
                }
            }
            "audio" => {
                let audio_channels = json["audio_channels"].as_str().and_then(|s| u8::from_str(s).ok()).unwrap();
                let audio_sample_rate = json["audio_sample_rate"].as_str().and_then(|s| u32::from_str(s).ok()).unwrap();
                FormatDetails::Audio {
                    audio_channels, audio_sample_rate
                }
            }
            _ => unreachable!()
        }
    }

    fn parse_mime(s: &str) -> (String, String, String) {
        let fs_index = s.find('/').unwrap();
        let format_type = s[0..fs_index].to_owned();
        let sc_index = s.find(';').unwrap();
        let extension = s[fs_index + 1..sc_index].to_owned();
        let cd_index = s.find("codecs=\"").unwrap() + 8;
        let codec = s[cd_index..s.len() - 1].to_owned();
        return (format_type, extension, codec);
    }
}

#[derive(Serialize, Debug)]
pub struct VideoDetails {
    channel_id: String,
    video_id: String,
    title: String,
    author: String,
    keywords: Vec<String>,
    average_rating: f32,
    short_description: String,
}

impl VideoDetails {
    fn parse_json(json: &Value) -> Self {
        let channel_id = json["channelId"].as_str().unwrap().to_owned();
        let video_id = json["videoId"].as_str().unwrap().to_owned();
        let title = json["title"].as_str().unwrap().to_owned();
        let author = json["author"].as_str().unwrap().to_owned();
        let average_rating = json["averageRating"].as_f64().unwrap() as f32;
        let keywords = serde_json::from_value::<Vec<String>>(json["keywords"].clone()).unwrap();
        let short_description = json["shortDescription"].as_str().unwrap().to_owned();

        Self {
            channel_id, video_id, title, author, average_rating, keywords, short_description
        }
    }
}

pub fn dump_to_file(file_name: &str, text: &str) {
    use std::io::Write;
    let mut file = std::fs::OpenOptions::new()
            .write(true)
            .create(true)
            .open(file_name).unwrap();
    file.set_len(0).unwrap();
    file.write(text.as_bytes()).unwrap();
}