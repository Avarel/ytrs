const YOUTUBE_VIDEO_INFO_URL: &str = "https://www.youtube.com/get_video_info";

use futures::{future::Future, stream::Stream};
use crate::errors::*;
use std::collections::HashMap;
use url::Url;

pub fn get_video_info(id: &str) -> Box<dyn Future<Item=VideoInfo, Error=Error> + Send> {
    let info_url = format!("{}?video_id={}", YOUTUBE_VIDEO_INFO_URL, id);

    let what = try_future!(info_url.parse::<hyper::Uri>());

    Box::new(crate::hyper_https::fetch_content(what).map(|content| {
        println!("{:?}", parse_query(&content));
        VideoInfo{}
    }).map_err(|e| e.into()))
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

fn parse_query(query_str: &str) -> HashMap<String, String> {
    let parse_query = url::form_urlencoded::parse(query_str.as_bytes());
    return parse_query.into_owned().collect::<HashMap<String, String>>();
}

pub struct VideoInfo {}