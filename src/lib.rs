extern crate pretty_env_logger;
extern crate rand;

use lazy_static;

use std::sync::{Arc, RwLock};
use std::env;

use rand::distributions::Alphanumeric;
use rand::Rng;
use regex::Regex;

use bytes::Bytes;
use kv::{Bucket, Config, Store};
use log::warn;

// db and bucket names
static TOKEN: &str = "token";
static VALUES: &str = "values";
lazy_static::lazy_static! {
    static ref DB: Arc<RwLock<Store>> = {
        // from 0.20.0 sled, default to 8mb
        let segsize = match env::var("SEGMENG_SIZE") {
            Ok(val) => val.parse().unwrap_or(2 << 22),
            Err(_) => 2 << 22
        };
        let cfg = Config {
            path: std::path::PathBuf::from("kv.db"),
            temporary: false,
            cache_capacity: None,
            use_compression: true,
            segment_size: Some(segsize),
            flush_every_ms: Some(30000),
        };
        let store = Store::new(cfg).unwrap();
        Arc::new(RwLock::new(store))
    };
}

fn open_bucket(name: &str, write: bool) -> Option<Bucket<String, String>> {
    if write {
        let db = DB.write().ok()?;
        db.bucket::<String, String>(Some(name)).ok()
    } else {
        let db = DB.read().ok()?;
        db.bucket::<String, String>(Some(name)).ok()
    }
}

pub fn token_exist(t: &str) -> bool {
    let token_key = format!("token-{}", t);
    if let Some(bucket) = open_bucket(TOKEN, false) {
        return bucket.contains(&token_key).ok().unwrap_or(true);
    }
    true
}

pub fn write_token(t: &str) -> bool {
    let token_key = format!("token-{}", t);
    if let Some(bucket) = open_bucket(TOKEN, true) {
        return bucket
            .set(&token_key, &token_key)
            .and_then(|_| Ok(true))
            .unwrap_or(true);
    }
    false
}

pub fn random_token() -> String {
    let len = 8;
    let mut token: String;

    loop {
        token = rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .take(len)
            .map(char::from)
            .collect();
        if !token_exist(&token) {
            break;
        }
    }
    token
}

pub fn get_value(t: String, k: String) -> String {
    let null = "".to_string();
    if !token_exist(&t) {
        return null;
    }

    if let Some(bucket) = open_bucket(VALUES, false) {
        let tk = format!("{}-{}", t, k);
        if let Ok(Some(val)) = bucket.get(&tk) {
            return val
        }
    }
    null
}

fn validate_key(key: &str) -> bool {
    lazy_static::lazy_static! {
        static ref RE: Regex = Regex::new(r"[0-9a-zA-Z]{1,100}").unwrap();
    }
    RE.is_match(key)
}

pub fn post_value(token: String, key: String, value: Bytes) -> String {
    let null = "".to_string();
    if !token_exist(&token) || !validate_key(&key) {
        return null;
    }

    let body = match String::from_utf8(value.to_vec()) {
        Ok(s) => s,
        Err(e) => {
            warn!("invalid utf8 string {}", e);
            return null;
        }
    };

    if let Some(bucket) = open_bucket(VALUES, true) {
        let tk = format!("{}-{}", token, key);
        if let Ok(_) = bucket.set(&tk, &body) {
            return key;
        }
    }
    null
}

pub fn list_keys(token: String) -> String {
    let null = "".to_string();
    if !token_exist(&token) {
        return null;
    }
    if let Some(bucket) = open_bucket(VALUES, false) {
        let pf = format!("{}-", token);
        let ks: Vec<String> = bucket
            .iter()
            .map(|k| match k {
                Ok(it) => {
                    match it.key::<String>() {
                        Ok(k) => k,
                        Err(_) => null.clone(),
                    }
                }
                Err(_) => null.clone(),
            })
            .filter(|x| x.starts_with(&pf))
            .map(|k| k.trim_start_matches(&pf).to_string())
            .collect();
        return "[".to_owned() + &ks.join(", ") + &"]".to_owned();
    }
    null
}

pub fn stats() -> String {
    let mut token_count = 0;
    let mut value_count = 0;
    if let Ok(db) = DB.read() {
        token_count = match db.bucket::<String, String>(Some(TOKEN)) {
            Ok(bucket) => bucket.iter().count(),
            Err(e) => {
                warn!("failed get tokens bucket {}", e);
                0
            }
        };

        value_count = match db.bucket::<String, String>(Some(VALUES)) {
            Ok(bucket) => bucket.iter().count(),
            Err(e) => {
                warn!("failed get values bucket {}", e);
                0
            }
        };
    }

    format!(
        "serving {} access token, {} k-v pairs",
        token_count, value_count
    )
}
