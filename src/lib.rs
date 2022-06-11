extern crate pretty_env_logger;
extern crate rand;

use lazy_static;

use std::sync::{Arc, RwLock};

use rand::distributions::Alphanumeric;
use rand::Rng;
use regex::Regex;

use bytes::Bytes;
use kv::{Config, Store, Bucket};
use log::warn;

// db and bucket names
static TOKEN: &str = "token";
static VALUES: &str = "values";
lazy_static::lazy_static! {
    static ref DB: Arc<RwLock<Store>> = {
        let cfg = Config {
            path: std::path::PathBuf::from("kv.db"),
            temporary: false,
            cache_capacity: None,
            use_compression: true,
            segment_size: Some(2 << 22), // old default 8mb
            flush_every_ms: Some(30000),
        };
        let store = Store::new(cfg).unwrap();
        Arc::new(RwLock::new(store))
    };
}

fn open_bucket(name: &str, write: bool) -> Option<Bucket<String, String>> {
    if write {
        if let Ok(db) = DB.write() {
            if let Ok(bucket) = db.bucket::<String, String>(Some(name)) {
                return Some(bucket)
            }
        }
    } else {
        if let Ok(db) = DB.read() {
            if let Ok(bucket) = db.bucket::<String, String>(Some(name)) {
                return Some(bucket);
            }
        }
    }
    None
}

pub fn token_exist(t: &str) -> bool {
    let token_key = format!("token-{}", t);
    if let Some(bucket) = open_bucket(TOKEN, false) {
        if let Ok(b) = bucket.contains(&token_key) {
            return b;
        }
    }
    true
}

pub fn write_token(t: &str) -> bool {
    let token_key = format!("token-{}", t);
    if let Some(bucket) = open_bucket(TOKEN, true) {
        if let Ok(_) = bucket.set(&token_key, &token_key) {
            return true;
        }
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
    if !token_exist(&token) || !validate_key(&key){
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

    format!("serving {} access token, {} k-v pairs", token_count, value_count)
}
