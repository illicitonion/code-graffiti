#[macro_use(try_future)]
extern crate boxfuture;
extern crate clap;
extern crate code_graffiti;
extern crate env_logger;
extern crate futures;
extern crate hyper;
#[macro_use]
extern crate log;
extern crate r2d2;
extern crate serde;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate serde_json;
extern crate serde_urlencoded;
extern crate tokio;

use boxfuture::{BoxFuture, Boxable};
use clap::{App, Arg};
use code_graffiti::github;
use futures::Future;
use std::net::SocketAddr;
use std::str::FromStr;

fn main() -> Result<(), String> {
    env_logger::init();

    let matches = app().get_matches();

    let address = SocketAddr::from_str(matches.value_of("addr").unwrap())
        .map_err(|e| format!("Bad addr passed: {}", e))?;

    let mut runtime = tokio::runtime::Runtime::new()
        .map_err(|e| format!("Failed to spawn tokio runtime: {}", e))?;

    let database = matches.value_of("database").unwrap();
    let database_manager = code_graffiti::make_connection_manager(database);
    let database_pool = r2d2::Pool::new(database_manager)
        .map_err(|e| format!("Failed to create r2d2 pool: {}", e))?;
    let github_client = github::Client::new();

    let server = hyper::Server::bind(&address)
        .serve(Server {
            database_pool,
            github_client,
        })
        .map_err(|e| {
            error!("{}", e);
            ()
        });

    runtime.spawn(server);
    runtime
        .shutdown_on_idle()
        .wait()
        .map_err(|()| "Error shutting down tokio runtime".to_owned())
}

#[derive(Clone)]
struct Server {
    database_pool: r2d2::Pool<code_graffiti::ConnectionManager>,
    github_client: github::Client,
}

mod get {
    extern crate boxfuture;
    extern crate code_graffiti;
    extern crate futures;

    use boxfuture::{BoxFuture, Boxable};
    use code_graffiti::github;
    use futures::Future;
    use std::borrow::Cow;

    #[derive(Debug, Deserialize)]
    pub struct Req {
        repo_user: String,
        repo_name: String,
        sha: Option<String>,
        #[serde(rename = "ref")]
        r: Option<String>,
        path: String,
        line: usize,
    }

    impl Req {
        pub fn repo(&self) -> code_graffiti::GithubRepository {
            code_graffiti::GithubRepository {
                user: &self.repo_user,
                name: &self.repo_name,
            }
        }

        pub fn into_line_in_repo(
            mut self,
            github_client: &github::Client,
        ) -> BoxFuture<code_graffiti::LineInRepo, String> {
            let sha_future = match (self.sha.take(), self.r.take()) {
                (Some(sha), None) => futures::future::ok(sha).to_boxed(),
                (None, Some(r)) => {
                    let cow = Cow::from(r);
                    github_client
                        .lookup_shalike(&self.repo(), cow)
                        .and_then(|maybe_sha| {
                            maybe_sha.ok_or_else(|| "Could not find ref".to_owned())
                        })
                        .to_boxed()
                }
                _ => {
                    return futures::future::err(
                        "Must specify exactly one of ref and sha".to_owned(),
                    ).to_boxed();
                }
            };

            sha_future
                .map(|sha| {
                    code_graffiti::LineInRepo {
                        // Guaranteed to be present based on above code picking from ref/sha.
                        sha: sha,
                        repo_user: self.repo_user,
                        repo_name: self.repo_name,
                        path: self.path,
                        line: self.line,
                    }
                })
                .to_boxed()
        }
    }
}

impl Server {
    fn get(&self, query: &str) -> BoxFuture<String, (String, hyper::StatusCode)> {
        let params: get::Req = try_future!(serde_urlencoded::from_str(query).map_err(|err| (
            format!("Error deserialising request params: {}", err),
            hyper::StatusCode::BAD_REQUEST,
        )));

        let context = params.into_line_in_repo(&self.github_client);

        let pool = self.database_pool.clone();
        context
            .map_err(|err| (err, hyper::StatusCode::NOT_FOUND))
            .and_then(move |context| {
                let connection = &*pool.get().map_err(|err| {
                    (
                        format!("Error getting database connection: {}", err),
                        hyper::StatusCode::INTERNAL_SERVER_ERROR,
                    )
                })?;
                code_graffiti::comments_for_line(connection, &context)
                    .map_err(|s| (s, hyper::StatusCode::INTERNAL_SERVER_ERROR))
            })
            .map(|result| json!(result).to_string())
            .to_boxed()
    }

    fn response_for(
        result: BoxFuture<String, (String, hyper::StatusCode)>,
    ) -> BoxFuture<hyper::Response<hyper::Body>, hyper::Error> {
        result
            .map(|body| hyper::Response::new(hyper::Body::from(body)))
            .or_else(|(message, status_code)| {
                let mut response = hyper::Response::new(hyper::Body::from(message));
                *response.status_mut() = status_code;
                Ok(response)
            })
            .to_boxed()
    }
}

impl hyper::service::Service for Server {
    type ReqBody = hyper::Body;
    type ResBody = hyper::Body;
    type Error = hyper::Error;
    type Future = BoxFuture<hyper::Response<Self::ResBody>, Self::Error>;

    fn call(&mut self, req: hyper::Request<hyper::Body>) -> Self::Future {
        match (req.method(), req.uri().path(), req.uri().query()) {
            (&hyper::Method::GET, "/", Some(query)) => Self::response_for(self.get(query)),
            _ => Self::response_for(Box::new(futures::future::err((
                String::new(),
                hyper::StatusCode::NOT_FOUND,
            )))),
        }
    }
}

impl hyper::service::NewService for Server {
    type ReqBody = hyper::Body;
    type ResBody = hyper::Body;
    type Error = hyper::Error;
    type Service = Self;
    type Future = futures::future::FutureResult<Self::Service, Self::InitError>;
    // TODO: Replace this with ! when it stabilises:
    // (See https://github.com/rust-lang/rust/issues/35121)
    type InitError = std::io::Error;

    fn new_service(&self) -> Self::Future {
        futures::future::ok(self.clone())
    }
}

impl futures::IntoFuture for Server {
    type Future = futures::future::FutureResult<Self::Item, Self::Error>;
    type Item = Self;
    // TODO: Replace this with ! when it stabilises:
    // (See https://github.com/rust-lang/rust/issues/35121)
    type Error = std::io::Error;

    fn into_future(self) -> Self::Future {
        futures::future::ok(self)
    }
}

fn app() -> App<'static, 'static> {
    App::new("Code Graffiti API")
        .arg(
            Arg::with_name("database")
                .short("d")
                .long("database")
                .takes_value(true)
                .required(true),
        )
        .arg(
            Arg::with_name("addr")
                .short("a")
                .long("addr")
                .takes_value(true)
                .required(true),
        )
}
