#[derive(Serialize, Deserialize, Default, Clone)]
pub struct Format<'a> {
    pub itag: i32,
    pub audio_bitrate: i32,
    pub extension: &'static str,
    pub resolution: &'static str,
    pub video_encoding: &'static str,
    pub audio_encoding: &'static str,
}

fn what() {
    Format {
        itag:          5,
        audio_bitrate:  64,
        extension:     "flv",
        resolution:    "240p",
        video_encoding: "Sorenson H.283",
        audio_encoding: "mp3",
    };
}