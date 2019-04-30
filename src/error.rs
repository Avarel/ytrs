#![allow(deprecated)]
error_chain! {
    foreign_links {
        Utf8Error(std::str::Utf8Error);
        HyperError(hyper::Error);
        IoError(std::io::Error);
        UriError(hyper::http::uri::InvalidUri);
    }
}

