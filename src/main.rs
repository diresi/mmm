use gethostname::gethostname;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::{env, error::Error, fs, io, path::PathBuf, vec::Vec};

const MMM: &str = "mmm.toml";

struct LineIter<I: Iterator<Item = String>> {
    iter: I,
    is_body: bool,
    any_header: bool,
    headers: HashSet<String>,
}

impl<T> Iterator for LineIter<T>
where
    T: Iterator<Item = String>,
{
    type Item = String;

    fn next(&mut self) -> Option<String> {
        loop {
            let val = self.iter.next();
            if val.is_none() {
                return None;
            }
            if self.is_body {
                return val;
            }

            let val = val.unwrap();
            if val.trim().is_empty() {
                self.is_body = true;
                // only emit empty line if headers have been emitted before
                if self.any_header {
                    return Some(val);
                }
            }

            if self.headers.contains("*") {
                return Some(val);
            }
            let header = val.split(":").next();
            if let Some(header) = header {
                if self.headers.contains(header) {
                    self.any_header = true;
                    return Some(val);
                }
            }
        }
    }
}

fn iter_text(headers: &HashSet<String>) -> impl Iterator<Item = String> {
    let iter = io::stdin().lines().filter_map(|s| s.ok());
    let headers = headers.clone();
    let iter = LineIter {
        iter,
        is_body: false,
        any_header: false,
        headers,
    };
    let prefix = vec!["```".to_string()];
    let suffix = vec!["```".to_string()];
    prefix.into_iter().chain(iter).chain(suffix.into_iter())
}

#[derive(Serialize, Deserialize, Debug)]
struct Config {
    url: Option<String>,
    username: Option<String>,
    headers: Option<HashSet<String>>,
}

impl Default for Config {
    fn default() -> Self {
        let url = "http://localhost".to_string();
        let username = gethostname();
        let username = username.to_string_lossy().to_string();
        Self {
            url: Some(url),
            username: Some(username),
            headers: Some(HashSet::from(["*".to_string()])),
        }
    }
}

fn load_cfg() -> Config {
    let mut cfg = Config::default();
    let p: PathBuf = ["/etc", MMM].iter().collect();
    let mut fps = vec![p];
    if let Some(mut p) = env::home_dir() {
        p.push(format!(".{}", MMM));
        fps.push(p);
    }
    for fp in fps.iter() {
        if let Ok(txt) = fs::read_to_string(fp) {
            if let Ok(d) = toml::from_str::<Config>(&txt) {
                if d.url.is_some() {
                    cfg.url = d.url;
                }
                if d.username.is_some() {
                    cfg.username = d.username;
                }
                if d.headers.is_some() {
                    cfg.headers = d.headers;
                }
            }
        }
    }
    cfg
}

fn main() -> Result<(), Box<dyn Error>> {
    let cfg = load_cfg();
    let payload = iter_text(&cfg.headers.unwrap())
        .collect::<Vec<String>>()
        .join("\n");

    let mut data = HashMap::new();
    if let Some(x) = cfg.username {
        data.insert("username", x);
    }
    data.insert("text", payload);

    if let Some(url) = cfg.url {
        let client = reqwest::blocking::Client::new();
        let _ = client.post(url).json(&data).send()?;
    }
    Ok(())
}
