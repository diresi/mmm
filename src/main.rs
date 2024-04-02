use gethostname::gethostname;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::{env, fs, io, vec::Vec, error::Error, path::PathBuf};

const MMM : &str = "mmm.toml";

fn iter_text() -> impl Iterator<Item = String> {
    let prefix = vec!["```".to_string()];
    let suffix = vec!["```".to_string()];
    let iter = io::stdin().lines().filter_map(|s| s.ok());
    prefix.into_iter().chain(iter).chain(suffix.into_iter())
}

#[derive(Serialize, Deserialize, Debug)]
struct Config {
    url: Option<String>,
    username: Option<String>,
}

impl Default for Config {
    fn default() -> Self {
        let url = "http://localhost".to_string();
        let username = gethostname();
        let username = username.to_string_lossy().to_string();
        Self {
            url: Some(url),
            username: Some(username),
        }
    }
}

fn load_cfg() -> Config {
    let mut cfg = Config::default();
    let p : PathBuf = ["/etc", MMM].iter().collect();
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
            }
        }
    }
    cfg
}


fn main() -> Result<(), Box<dyn Error>> {
    let cfg = load_cfg();
    let payload = iter_text().collect::<Vec<String>>().join("\n");

    let mut data = HashMap::new();
    if let Some(x) = cfg.username {
        data.insert("username", x);
    }
    data.insert("text", payload);

    if let Some(url) = cfg.url {
        let client = reqwest::blocking::Client::new();
        let _ = client
            .post(url)
            .json(&data)
            .send()?;
    }
    Ok(())
}
