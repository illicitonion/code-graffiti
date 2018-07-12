#[macro_use(value_t)]
extern crate clap;
extern crate code_graffiti;
extern crate env_logger;
extern crate futures;

use clap::{App, Arg, SubCommand};
use code_graffiti::github;
use futures::Future;
use std::borrow::Cow;

fn main() -> Result<(), String> {
    env_logger::init();

    let matches = app().get_matches();

    let connection = code_graffiti::connect(matches.value_of("database").unwrap())?;
    let repo = code_graffiti::GithubRepository {
        user: matches.value_of("repo-user").unwrap(),
        name: matches.value_of("repo-name").unwrap(),
    };
    let raw_ref = Cow::from(matches.value_of("ref").unwrap());
    let sha = github::Client::new()
        .lookup_shalike(&repo, raw_ref.clone())
        .wait()?
        .ok_or_else(|| format!("Count not find sha {}", raw_ref))?;
    let line = code_graffiti::LineInRepo {
        repo_user: repo.user.to_owned(),
        repo_name: repo.name.to_owned(),
        path: matches.value_of("file").unwrap().to_owned(),
        sha: sha,
        line: value_t!(matches.value_of("line"), usize).unwrap(),
    };

    match matches.subcommand() {
        ("get", Some(_)) => {
            let comments = code_graffiti::comments_for_line(&connection, &line)?;
            if comments.is_empty() {
                println!("[No comments]");
            }
            for (index, comment) in comments.iter().enumerate() {
                println!("Comment {}: {}", index, comment);
            }
        }
        ("post", Some(sub_match)) => code_graffiti::leave_comment(
            &connection,
            &line,
            sub_match.value_of("comment").unwrap(),
        )?,
        _ => {
            app().print_long_help().unwrap();
        }
    }
    Ok(())
}

fn app() -> App<'static, 'static> {
    App::new("Code Graffiti CLI")
        .arg(
            Arg::with_name("database")
                .short("d")
                .long("database")
                .takes_value(true)
                .required(true),
        )
        .arg(
            Arg::with_name("repo-user")
                .short("u")
                .long("repo-user")
                .takes_value(true)
                .required(true),
        )
        .arg(
            Arg::with_name("repo-name")
                .short("r")
                .long("repo-name")
                .takes_value(true)
                .required(true),
        )
        .arg(
            Arg::with_name("file")
                .short("f")
                .long("file")
                .takes_value(true)
                .required(true),
        )
        .arg(
            Arg::with_name("ref")
                .short("s")
                .long("ref")
                .takes_value(true)
                .required(true),
        )
        .arg(
            Arg::with_name("line")
                .short("l")
                .long("line")
                .takes_value(true)
                .required(true),
        )
        .subcommand(SubCommand::with_name("get"))
        .subcommand(
            SubCommand::with_name("post")
                .arg(Arg::with_name("comment").takes_value(true).required(true)),
        )
}
