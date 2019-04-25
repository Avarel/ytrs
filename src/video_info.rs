use futures::{future::Future, stream::Stream};
use serde::{Serialize, Deserialize};

use crate::errors::*;

const YOUTUBE_VIDEO_INFO_URL: &str = "https://www.youtube.com/get_video_info";

pub fn get_video_info(id: &str) -> impl Future<Item=VideoInfo, Error=Error> {
    let info_url = format!("{}?video_id={}", YOUTUBE_VIDEO_INFO_URL, id);

    crate::hyper_https::fetch_content(info_url.parse().unwrap()).map(|content| {
        dump_to_file("dump2.json", &serde_json::to_string_pretty(&serde_urlencoded::from_str::<serde_json::Value>(&content).unwrap()).unwrap());
        serde_urlencoded::from_str(&content).unwrap()
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
    #[serde(deserialize_with = "deserialize_spec")]
    adaptive_fmts: Vec<Format>,
    #[serde(deserialize_with = "from_str")]
    player_response: serde_json::Value,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PlayerResponse {
    #[serde(rename = "videoDetails")]
    video_details: VideoDetails
}

#[derive(Serialize, Deserialize, Debug)]
pub struct VideoDetails {
    #[serde(rename = "channelId")]
    channel_id: String,
    #[serde(rename = "videoId")]
    video_id: String,
    title: String,
    author: String,
    #[serde(rename = "averageRating")]
    average_rating: f32,
    keywords: Vec<String>,
    #[serde(rename = "shortDescription")]
    short_description: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(untagged)]
pub enum FormatType {
    Video {
        #[serde(deserialize_with = "from_str")]
        fps: u16,
        size: String,
        quality_label: String
    },
    Audio {
        #[serde(deserialize_with = "from_str")]
        audio_channels: u8,
        #[serde(deserialize_with = "from_str")]
        audio_sample_rate: u32
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Format {
    #[serde(deserialize_with = "from_str")]
    itag: u32,
    #[serde(deserialize_with = "from_str")]
    bitrate: u32,
    url: String,
    #[serde(flatten)]
    specific: FormatType,
    #[serde(rename = "type", deserialize_with = "deserialize_mime")]
    mime_type: MimeType
}

pub fn dump_to_file(file_name: &str, text: &str) {
    use std::io::Write;
    let mut file = std::fs::OpenOptions::new()
            .write(true)
            .create(true)
            .open(file_name).unwrap();
    file.write(text.as_bytes()).unwrap();
}

#[derive(Serialize, Debug)]
pub struct MimeType {
    format_type: String,
    extension: String,
    codec: String
}

fn deserialize_mime<'de, D>(deserializer: D) -> std::result::Result<MimeType, D::Error>
    where D: serde::de::Deserializer<'de>
{
    let s = String::deserialize(deserializer)?;
    
    let fs_index = s.find('/').unwrap();
    let format_type = s[0..fs_index].to_owned();
    let sc_index = s.find(';').unwrap();
    let extension = s[fs_index + 1..sc_index].to_owned();
    let cd_index = s.find("codecs=\"").unwrap() + 8;
    let codec = s[cd_index..s.len() - 1].to_owned();
    return Ok(MimeType { format_type, extension, codec });
}

// fn json_string<'de, T, D>(deserializer: D) -> std::result::Result<T, D::Error>
//     where T: serde::de::DeserializeOwned,
//           D: serde::de::Deserializer<'de>
// {
//     let s = <&str>::deserialize(deserializer)?;
//     serde_json::from_str::<T>(s).map_err(serde::de::Error::custom)
// }

// fn urlencoded_string<'de, T, D>(deserializer: D) -> std::result::Result<T, D::Error>
//     where T: serde::de::DeserializeOwned,
//           D: serde::de::Deserializer<'de>
// {
//     let s = <&str>::deserialize(deserializer)?;
//     serde_urlencoded::from_str::<T>(s).map_err(serde::de::Error::custom)
// }

// fn deserialize_json_string<'de, T, D>(deserializer: D) -> std::result::Result<T, D::Error>
//     where T: serde::de::DeserializeOwned,
//           D: serde::de::Deserializer<'de>,
// {
//     struct JsonStringVisitor<T> {
//         phantom: std::marker::PhantomData<T>
//     }

//     impl<'de, T> serde::de::Visitor<'de> for JsonStringVisitor<T> where T: serde::de::DeserializeOwned {
//         type Value = T;

//         fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
//             formatter.write_str("a string containing json data")
//         }
    
//         fn visit_str<E>(self, v: &str) -> std::result::Result<Self::Value, E>
//         where E: serde::de::Error
//         {
//             serde_json::from_str::<T>(v).map_err(E::custom)
//         }
//     }
    
//     deserializer.deserialize_any(JsonStringVisitor::<T> { phantom: std::marker::PhantomData })
// }

// fn deserialize_urlencoded_string<'de, T, D>(deserializer: D) -> std::result::Result<T, D::Error>
//     where T: serde::de::DeserializeOwned,
//           D: serde::de::Deserializer<'de>,
// {
//     struct JsonStringVisitor<T> {
//         phantom: std::marker::PhantomData<T>
//     }

//     impl<'de, T> serde::de::Visitor<'de> for JsonStringVisitor<T> where T: serde::de::DeserializeOwned {
//         type Value = T;

//         fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
//             formatter.write_str("a string containing json data")
//         }
    
//         fn visit_str<E>(self, v: &str) -> std::result::Result<Self::Value, E>
//         where E: serde::de::Error
//         {
//             serde_urlencoded::from_str::<T>(v).map_err(E::custom)
//         }
//     }
    
//     deserializer.deserialize_any(JsonStringVisitor::<T> { phantom: std::marker::PhantomData })
// }

fn deserialize_spec<'de, D>(deserializer: D) -> std::result::Result<Vec<Format>, D::Error>
    where D: serde::de::Deserializer<'de>,
{
    struct JsonStringVisitor;
    impl<'de> serde::de::Visitor<'de> for JsonStringVisitor {
        type Value = Vec<Format>;

        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            formatter.write_str("a string containing json data")
        }
    
        fn visit_str<E>(self, v: &str) -> std::result::Result<Self::Value, E>
        where E: serde::de::Error
        {
            v.split(',').map(|s| serde_urlencoded::from_str(s)).collect::<std::result::Result<Vec<_>, _>>().map_err(E::custom)
        }
    }
    
    deserializer.deserialize_any(JsonStringVisitor)
}

fn from_str<'de, T, D>(deserializer: D) -> std::result::Result<T, D::Error>
    where T: std::str::FromStr,
          T::Err: std::fmt::Display,
          D: serde::de::Deserializer<'de>
{
    let s = String::deserialize(deserializer)?;
    T::from_str(&s).map_err(serde::de::Error::custom)
}