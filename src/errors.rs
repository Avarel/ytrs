#![allow(deprecated)]

#[macro_export]
macro_rules! try_future {
    ($e:expr) => {
        {
            let temp = $e;
            if temp.is_err() {
                return Box::new(futures::future::err(temp.unwrap_err().into()));
            }
            temp.unwrap()
        }
    };
}

error_chain! {
    foreign_links {
        Utf8Error(std::str::Utf8Error);
        UrlParseError(url::ParseError);
        HyperError(hyper::Error);
        IoError(std::io::Error);
        UriError(hyper::http::uri::InvalidUri);
    }
}

