#[macro_use] extern crate tracing;

use std::{collections::HashMap, io::{stdin, BufRead}};

use clap::Parser;
use chrono::{Utc, DateTime, TimeZone};
use cli::Cli;

use regex::Regex;
use serde_with::{serde_as, StringWithSeparator, formats::CommaSeparator};

use tracing::Level;
use tracing_subscriber::fmt::{self, time};

mod cli;

#[tokio::main]
async fn main() {
    std::panic::set_hook(Box::new(tracing_panic::panic_hook));

    let cli = Cli::parse();

    let level = match cli.debug {
        0 => Level::WARN,
        1 => Level::INFO,
        2 => Level::DEBUG,
        _ => Level::TRACE,
    };

    let format = fmt::format()
        .with_level(true)
        .with_target(false)
        .with_thread_names(false)
        .without_time()
        .compact();

    tracing_subscriber::fmt()
        .with_max_level(level)
        .with_timer(time::Uptime::default())
        .event_format(format)
        .with_writer(std::io::stderr)
        .init();

    let password = if cli.password_stdin {
        let stdin = stdin();
        stdin.lock().lines().next()
            .expect("No password provided via stdin")
            .expect("Stdin could not be read for password")
    }
    else if let Some(x) = cli.password {
        x
    }
    else {
        panic!("No reddit password was provided")
    };

    let client = reqwest::ClientBuilder::new()
        .cookie_store(true)
        .user_agent("Mozilla/5.0 (Windows NT 10.0; rv:112.0) Gecko/20100101 Firefox/112.0")
        .build()
        .expect("Failed to build HTTP client");

    // Get CSRF token
    let response = client.get("https://www.reddit.com/login").send().await
        .expect("Failed to get Reddit login");

    let body = response.text().await.unwrap();
    let token_regex = Regex::new(r#"name="csrf_token" value="([^"]+)""#).unwrap();
    let token_cap = token_regex.captures(&body).expect("Unable to find csrf_token");
    let csrf_token = token_cap.get(1).expect("Unable to extract csrf_token").as_str().to_string();

    // Perform login
    let response = client.post("https://www.reddit.com/login")
        .form(&LoginData {
            username: cli.username,
            password,
            csrf_token,
            dest: String::from("https://www.reddit.com"),
            otp: String::from(""),
        })
        .send().await
        .expect("Failed to get Reddit login");

    assert!(response.status().is_success(), "Failed to login: {}",
        response.text().await.unwrap_or_else(|_| String::from("No body")));

    let mut aggregate = vec![];

    let mut position = String::new();

    loop {
        let _span = span!(Level::INFO, "fetch_loop", url = format!("https://www.reddit.com/saved.json?after={}", position)).entered();
        // Fetch saved
        let response = client.get("https://www.reddit.com/saved.json")
            .query(&HashMap::from([("after", position)])).send().await
            .expect("Unable to make saved posts request");

        let response = response.text().await
            .expect("Failed to fetch saved posts body");

        let response: RedditResponse = serde_json::from_str(&response)
            .expect(&format!("Failed to parse response: {}", response));

        let RedditResponse::Listing(listing) = response else {
            panic!("Failed traversing response, expecting list, got: {:?}", response);
        };

        for post in listing.children {
            match post {
                RedditResponse::Post(post) => {
                    aggregate.push(ArchiveItem {
                        title: post.title,
                        url: post.url,
                        saved: Utc.timestamp_opt(post.created as _, 0).latest().unwrap(),
                        tags: vec![format!("r/{}", post.subreddit), String::from("reddit")],
                    })
                }
                RedditResponse::Comment(comment) => {
                    aggregate.push(ArchiveItem {
                        title: comment.link_title,
                        url: comment.link_permalink,
                        saved: Utc.timestamp_opt(comment.created as _, 0).latest().unwrap(),
                        tags: vec![format!("r/{}", comment.subreddit), String::from("reddit")],
                    })
                }
                o => unreachable!("Lists should only include posts and comments, got: {:?}", o),
            };
        }

        if let Some(after) = listing.after {
            position = after;
        }
        else {
            break;
        }
    }

    println!("{}", serde_json::to_string(&aggregate).unwrap());
}

#[derive(Clone, Debug, serde::Deserialize)]
#[serde(tag = "kind", content = "data")]
enum RedditResponse {
    Listing(RedditListing),
    #[serde(alias = "t1")] Comment(RedditComment),
    #[serde(alias = "t3")] Post(RedditPost),
}

#[derive(Clone, Debug, serde::Deserialize)]
struct RedditPost {
    title: String,
    created: f64,
    url: String,
    subreddit: String,
}

#[derive(Clone, Debug, serde::Deserialize)]
struct RedditComment {
    link_title: String,
    link_permalink: String,
    created: f64,
    subreddit: String,
}

#[derive(Clone, Debug, serde::Deserialize)]
struct RedditListing {
    after: Option<String>,
    #[serde(alias = "dist")] count: i32,
    children: Vec<RedditResponse>,
}

#[serde_as]
#[derive(serde::Serialize)]
struct ArchiveItem {
    title: String,
    url: String,
    saved: DateTime<Utc>,
    #[serde_as(as = "StringWithSeparator::<CommaSeparator, String>")]
    tags: Vec<String>,
}

#[derive(serde::Serialize)]
struct LoginData {
    csrf_token: String,
    username: String,
    password: String,
    dest: String,
    otp: String,
}
