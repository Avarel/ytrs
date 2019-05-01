use serde::Serialize;
use crate::error::Result as CrateResult;

const YOUTUBE_VIDEO_INFO_URL: &str = "https://www.youtube.com/get_video_info";

pub async fn get_video_info(id: &str) -> CrateResult<VideoInfo> {
    let info_url = format!("{}?video_id={}", YOUTUBE_VIDEO_INFO_URL, id);
    let content = await!(crate::hyper_https::fetch_content(info_url.parse().unwrap()))?;
    let info = VideoInfo::from_json(serde_urlencoded::from_str(&content).unwrap()).unwrap();
    Ok(info)
}

/// Get the YouTube video ID from a link.
pub fn get_id_from_string(s: &str) -> CrateResult<String> {
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

/// General video container.
#[derive(Serialize, Debug)]
pub struct VideoInfo {
    /// Duration of the video in seconds.
    pub length_seconds: u64,
    /// Various formats that YouTube offers for the video.
    /// Each format contains an extension, link, and more information for the specific video file.
    pub formats: Vec<Format>,
    /// Detailed video information such as keywords, title, and more.
    pub details: VideoDetails
}

/// Detailed video information.
#[derive(Serialize, Debug)]
pub struct VideoDetails {
    /// YouTube ID for the channel of the `author`.
    pub channel_id: String,
    /// The name of the channel for this video.
    pub author: String,
    /// YouTube ID for this video.
    pub video_id: String,
    /// The title of the video.
    pub title: String,
    /// Keywords of the video, may be empty.
    pub keywords: Vec<String>,
    /// Average rating of the video.
    pub average_rating: f32,
    /// A short description of the video.
    pub short_description: String,
    /// View count of the video
    pub view_count: u64,
}

/// Specific format details, depending on if the content is a video or audio stream.
#[derive(Serialize, Debug)]
#[serde(tag = "type")]
pub enum FormatDetails {
    Video {
        /// Frames per second, or how smooth the video is.
        fps: u16,
        /// Size and dimension of the video, generally in the form of `(height)x(width)`, ie: `1920x1080`.
        size: String,
        /// Quality of the video, generally in the form of `(resolution)p`, ie: `720p`.
        quality_label: String
    },
    Audio {
        /// How many audio channels there are.
        audio_channels: u8,
        /// Sample rate of the audio.
        audio_sample_rate: u32
    }
}

/// General format details.
#[derive(Serialize, Debug)]
pub struct Format {
    /// Specific YouTube identifier for this format, formats with the same `itag` should also have
    /// the same `codec`, `extension`, `bitrate`, and `details`.
    pub itag: u32,
    /// Bitrate of the format.
    pub bitrate: u32,
    /// The direct download link for the file.
    pub url: String,
    /// The extension of the file.
    pub extension: String,
    /// Encoding standard.
    pub codec: String,
    /// Specific format details.
    pub details: FormatDetails
}

use serde_json::Value;
use std::str::FromStr;
impl VideoInfo {
    #[doc(hidden)]
    fn from_json(json: Value) -> CrateResult<Self> {
        // let context_version = find_str(&json, "innertube_context_client_version")?;
        // if context_version != "1.20190423" {
        //     return Err("API context version out of date".into())
        // }

        let length_seconds = parse_str(&json, "length_seconds", u64::from_str)?;

        let formats = find_str(&json, "adaptive_fmts")?
            .split(',')
            .map(|s| serde_urlencoded::from_str::<Value>(s))
            .collect::<Result<Vec<_>, _>>()
            .map_err(|_| "Parsing error")?
            .iter()
            .map(|j| Format::parse_json(j))
            .collect::<CrateResult<Vec<_>>>()?;

        let player_response = parse_str(&json, "player_response", |s| serde_json::from_str::<Value>(s))?;
        let details = VideoDetails::parse_json(&player_response["videoDetails"])?;

        Ok(Self {
            length_seconds,
            formats,
            details
        })
    }
}

impl Format {
    /// Get the download interface foor a video information.
    // #[cfg(feature + "download")]
    pub fn download(&self) -> crate::download::DownloadFormat {
        crate::download::DownloadFormat { format: &self }
    }

    #[doc(hidden)]
    fn parse_json(json: &Value) -> CrateResult<Format> {
        let itag = parse_str(json, "itag", u32::from_str)?;
        let bitrate = parse_str(json, "bitrate", u32::from_str)?;
        let url = find_string(json, "url")?;
        let (format_type, extension, codec) = json["type"].as_str().map(Self::parse_mime).ok_or("mime")??;
        let details = Self::parse_details(&format_type, json)?;

        Ok(Self {
            itag,
            bitrate,
            url,
            extension,
            codec,
            details,
        })
    }

    #[doc(hidden)]
    fn parse_details(format_type: &str, json: &Value) -> CrateResult<FormatDetails> {
        match format_type {
            "video" => {
                let fps = parse_str(json, "fps", u16::from_str)?;
                let size = find_string(json, "size")?;
                let quality_label = find_string(json, "quality_label")?;
                Ok(FormatDetails::Video {
                    fps, size, quality_label
                })
            }
            "audio" => {
                let audio_channels = parse_str(json, "audio_channels", u8::from_str)?;
                let audio_sample_rate = parse_str(json, "audio_sample_rate", u32::from_str)?;
                Ok(FormatDetails::Audio {
                    audio_channels, audio_sample_rate
                })
            }
            _ => unreachable!()
        }
    }

    #[doc(hidden)]
    fn parse_mime(s: &str) -> CrateResult<(String, String, String)> {
        let fs_index = s.find('/').ok_or("index of \"/\" not found")?;
        let format_type = s[0..fs_index].to_owned();
        let sc_index = s.find(';').ok_or("index of \";\" not found")?;
        let extension = s[fs_index + 1..sc_index].to_owned();
        let cd_index = s.find("codecs=\"").ok_or("index of \"codecs=\" not found")? + 8;
        let codec = s[cd_index..s.len() - 1].to_owned();
        return Ok((format_type, extension, codec));
    }
}

impl VideoDetails {
    #[doc(hidden)]
    fn parse_json(json: &Value) -> CrateResult<Self> {
        let channel_id = find_string(json, "channelId")?;
        let video_id = find_string(json, "videoId")?;
        let title = find_string(json, "title")?;
        let author = find_string(json, "author")?;
        let average_rating = json["averageRating"].as_f64().ok_or("Missing field \"averageRating\"")? as f32;
        let view_count = parse_str(json, "viewCount", u64::from_str)?;

        let keywords = {
            let keywords_json = json["keywords"].clone();
            if keywords_json.is_null() {
                Vec::new()
            } else {
                serde_json::from_value(keywords_json).unwrap()
            }
        };

        let short_description = find_string(json, "shortDescription")?;

        Ok(Self {
            channel_id, video_id, title, author, average_rating, keywords, short_description, view_count
        })
    }
}

#[doc(hidden)]
fn find_str<'a>(json: &'a Value, k: &str) -> CrateResult<&'a str> {
    json[k].as_str().ok_or(format!("Missing field \"{}\"", k).into())
}

#[doc(hidden)]
fn find_string(json: &Value, k: &str) -> CrateResult<String> {
    find_str(json, k).map(|s| s.to_owned())
}

#[doc(hidden)]
fn parse_str<T, X, F: FnOnce(&str) -> Result<T, X>>(json: &Value, k: &str, f: F) -> CrateResult<T> {
    find_str(json, k).and_then(|s| f(s).map_err(|_| "Parsing error".into()))
}