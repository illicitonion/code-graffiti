use boxfuture::{BoxFuture, Boxable};
use futures::{self, Future, Stream};
use hyper;
use hyper_tls;
use models::GithubRepository;
use regex;
use std::borrow::{Borrow, Cow};

#[derive(Clone)]
pub struct Client {
    client: hyper::Client<hyper_tls::HttpsConnector<hyper::client::HttpConnector>, hyper::Body>,
}

impl Client {
    pub fn new() -> Client {
        let https = hyper_tls::HttpsConnector::new(4).unwrap();
        let client = hyper::Client::builder().build::<_, hyper::Body>(https);
        Client { client: client }
    }

    pub fn lookup_shalike(
        &self,
        repo: &GithubRepository,
        shalike: Cow<str>,
    ) -> BoxFuture<Option<String>, String> {
        if is_sha(shalike.borrow()) {
            return futures::future::ok(Some(shalike.into_owned())).to_boxed();
        }
        info!("Looking up sha {} for repo {}", shalike, repo);

        let mut req = hyper::Request::new(hyper::Body::empty());
        *req.uri_mut() = try_future!(
            format!("https://api.github.com/repos/{}/commits/{}", repo, shalike)
                .parse()
                .map_err(|err| format!("Bad github url: {}", err))
        );
        req.headers_mut().insert(
            "Accept",
            hyper::header::HeaderValue::from_str("application/vnd.github.VERSION.sha").unwrap(),
        );
        req.headers_mut().insert(
            "User-Agent",
            hyper::header::HeaderValue::from_str("illicitonion").unwrap(),
        );

        self.client
            .request(req)
            .map_err(|err| format!("Error requesting sha from github: {}", err))
            .and_then(|res| {
                let status = res.status();
                res.into_body()
                    .concat2()
                    .map_err(|err| format!("Error reading response body: {}", err))
                    .and_then(|b| {
                        String::from_utf8(b.to_vec())
                            .map_err(|err| format!("Response not utf8: {}", err))
                    })
                    .and_then(move |body| match status {
                        hyper::StatusCode::OK => Ok(Some(body)),
                        hyper::StatusCode::UNPROCESSABLE_ENTITY => Ok(None),
                        _ => Err(format!(
                            "Bad status code from github: {:?}: {}",
                            status, body
                        )),
                    })
            })
            .to_boxed()
    }
}

lazy_static! {
    static ref SHA_REGEX: regex::Regex = regex::Regex::new("^[0-9a-fA-f]{40}$").unwrap();
}

fn is_sha(maybe_sha: &str) -> bool {
    maybe_sha.len() == 40 && SHA_REGEX.is_match(maybe_sha)
}
