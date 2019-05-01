use tokio::prelude::*;
use crate::hyper_https::get_client;
use crate::error::Result;
use crate::video_info::Format;

/// Downloading utilities wrapper for `Format`.
pub struct DownloadFormat<'f> {
    pub format: &'f Format
}

impl<'a> DownloadFormat<'a> {
    /// Download the whole stream as a file into a directory at path `p`.
    /// Returns an error if `self.open_stream()` also errors, or if there is an IO exception.
    pub async fn to_dir(self, name: &'a str, p: &'a std::path::Path) -> Result<std::fs::File> {
        let p = p.join(format!("{}.{}", name, self.format.extension));
        
        let file = std::fs::OpenOptions::new()
            .write(true)
            .create(true)
            .open(p)?;

        if file.metadata()?.is_file() {
            bail!(format!("{:?} is a file, to_dir can not download to a file", file))
        }

        file.set_len(0)?;

        await!(self.to_file(file))
    }

    /// Download the whole stream into a file.
    /// Returns an error if `self.open_stream()` also errors, or if there is an IO exception.
    pub async fn to_file(self, file: std::fs::File) -> Result<std::fs::File> {
        let mut file = tokio::fs::File::from_std(file);

        let (mut stream, _) = await!(self.open_stream())?;

        while let Some(chunk) = await!(stream.next()) {
            await!(tokio::io::write_all(&mut file, chunk?))?;
        }

        Ok(file.into_std())
    }

    /// Returns a `(hyper::Body, u64)` where `hyper::Body` is a `Stream`, and `u64` 
    /// is the content length header of the response.
    /// Returns an error if it fails to connect (status code != 200) or ContentLength header isn't present/.
    pub async fn open_stream(self) -> Result<(hyper::Body, u64)> {
        let res = await!(get_client().get(self.url()))?;

        if res.status() != 200 {
            bail!("Failed to connect, status: {}", res.status());
        }

        let len = res.headers().get(hyper::header::CONTENT_LENGTH)
            .ok_or("Expected Content-Length header")?
            .to_str()
            .unwrap()
            .parse::<u64>()
            .map_err(|_| "Failed to parse Content-Length header")?;

        Ok((res.into_body(), len))
    }

    /// Get a `hyper::Uri` instance.
    pub fn url(&self) -> hyper::Uri {
        self.format.url.parse::<hyper::Uri>().unwrap()
    }
}
