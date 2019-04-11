const YOUTUBE_VIDEO_INFO_URL: &str = "https://www.youtube.com/get_video_info";

use crate::errors::*;
use url::Url;

pub fn get_video_info_from_string(value: &str) -> Result<VideoInfo> {
    let parse_url = value.parse::<Url>()?;

    if parse_url.host_str() == Some("youtu.be") {
        unimplemented!("get video from short url");
        // return get_video_info_from_short_url(&parse_url);
    }

    get_video_info_from_url(&parse_url)
}

fn get_video_info_from_url(u: &Url) -> Result<VideoInfo> {
    unimplemented!()
}

fn get_video_info(id: &str) -> Result<()> {
    let info_url = format!("{}?video_id={}", YOUTUBE_VIDEO_INFO_URL, id);

    let mut resp = crate::hyper_https::get_client().get(info_url.parse::<hyper::Uri>()?);
//    if resp.status() != hyper::StatusCode(200) {
//        bail!("video info response invalid status code");
//    }

    unimplemented!()
}

pub fn get_id_from_string(s: &str) -> Result<String> {
    get_id_from_url(&s.parse::<Url>()?)
}

fn get_id_from_url(u: &Url) -> Result<String> {
    if let Some(video_id) = u.query_pairs().find(|p| p.0 == "v") {
        return Ok(video_id.1.into_owned());
    }
    bail!("YouTube video id not found")
}

pub struct VideoInfo {}