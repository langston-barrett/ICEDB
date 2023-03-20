// This code was taken from Glacier.
//
// Permission is hereby granted, free of charge, to any
// person obtaining a copy of this software and associated
// documentation files (the "Software"), to deal in the
// Software without restriction, including without
// limitation the rights to use, copy, modify, merge,
// publish, distribute, sublicense, and/or sell copies of
// the Software, and to permit persons to whom the Software
// is furnished to do so, subject to the following
// conditions:
//
// The above copyright notice and this permission notice
// shall be included in all copies or substantial portions
// of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF
// ANY KIND, EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED
// TO THE WARRANTIES OF MERCHANTABILITY, FITNESS FOR A
// PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT
// SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY
// CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION
// OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR
// IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER
// DEALINGS IN THE SOFTWARE.

use once_cell::sync::{Lazy, OnceCell};
use regex::Regex;
use reqwest::blocking::Client;
use serde_json::Value;
use std::env::{var, VarError};

static CLIENT: Lazy<Client> = Lazy::new(|| {
    Client::builder()
        .user_agent("rust-lang/glacier")
        .build()
        .unwrap()
});

pub(crate) struct Config {
    token: String,
}

impl Config {
    pub(crate) fn from_env() -> Result<Self, VarError> {
        Ok(Self {
            token: var("GITHUB_TOKEN")?,
        })
    }
}

pub(crate) fn get_labeled_issues(
    config: &Config,
    repo: &str,
    label_name: String,
) -> Result<Vec<Value>, reqwest::Error> {
    let url = format!("https://api.github.com/repos/{repo}/issues?state=all&labels={label_name}");

    let mut issues: Vec<Value> = CLIENT
        .get(&url)
        .bearer_auth(&config.token)
        .send()?
        .error_for_status()?
        .json()?;

    let pages = get_result_length(config, &url).unwrap();

    if pages > 1 {
        for i in 2..=pages {
            // Make sure we don't run into rate limiting...
            std::thread::sleep(std::time::Duration::from_secs(5));
            let url = format!(
                "https://api.github.com/repos/{repo}/issues?state=all&labels={label_name}&page={i}"
            );
            let mut paged_issues: Vec<Value> = CLIENT
                .get(&url)
                .bearer_auth(&config.token)
                .send()?
                .error_for_status()?
                .json()?;

            issues.append(&mut paged_issues);
        }
    }

    Ok(issues)
}

fn get_result_length(config: &Config, url: &str) -> Result<usize, Box<dyn std::error::Error>> {
    static RE_LAST_PAGE: OnceCell<Regex> = OnceCell::new();
    let res = CLIENT.get(url).bearer_auth(&config.token).send()?;

    if res.status().is_success() {
        if let Some(link) = res.headers().get("Link") {
            let link_string = String::from_utf8(link.as_bytes().to_vec()).unwrap();
            let re_last_page =
                RE_LAST_PAGE.get_or_init(|| Regex::new(r#"page=([0-9]+)>; rel="last""#).unwrap());
            let last_page_number = re_last_page
                .captures(&link_string)
                .unwrap()
                .get(1)
                .unwrap()
                .as_str();
            let pages: usize = last_page_number.parse().unwrap();

            Ok(pages)
        } else {
            Ok(0)
        }
    } else {
        Ok(0)
    }
}
