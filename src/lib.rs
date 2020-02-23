extern crate pretty_env_logger;
extern crate rand;

use lazy_static;

use std::iter;
use std::sync::{Arc, RwLock};

use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};
use regex::Regex;

use bytes::Bytes;
use kv::{Config, Store};
use log::{warn};

// db and bucket names
static TOKEN: &str = "token";
static VALUES: &str = "values";
lazy_static::lazy_static! {
    static ref DB: Arc<RwLock<Store>> = {
        let cfg = Config {
            path: std::path::PathBuf::from("kv.db"),
            read_only: false,
            temporary: false,
            use_compression: true,
            flush_every_ms: Some(30000),
        };
        let store = Store::new(cfg).unwrap();
        Arc::new(RwLock::new(store))
    };
}

pub fn token_exist(t: &str) -> bool {
    let token_key = format!("token-{}", t);
    if let Ok(db) = DB.read() {
        if let Ok(bucket) = db.bucket::<String, String>(Some(TOKEN)) {
            if let Ok(b) = bucket.contains(&token_key) {
                return b;
            }
        }
    }
    true
}

pub fn write_token(t: &str) -> bool {
    let token_key = format!("token-{}", t);
    if let Ok(db) = DB.write() {
        if let Ok(bucket) = db.bucket::<String, String>(Some(TOKEN)) {
            if let Ok(_) = bucket.set(token_key, "1") {
                return true;
            }
        }
    }
    false
}

pub fn random_token() -> String {
    let len = 8;
    let mut token: String;

    loop {
        let mut rng = thread_rng();
        token = iter::repeat(())
            .map(|()| rng.sample(Alphanumeric))
            .take(len)
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

    let db = match DB.read() {
        Ok(d) => d,
        Err(e) => {
            warn!("failed get read lock {}", e);
            return null;
        }
    };

    let bucket = match db.bucket::<String, String>(Some(VALUES)) {
        Ok(b) => b,
        Err(e) => {
            warn!("failed get values bucket {}", e);
            return null;
        }
    };

    let tk = format!("{}-{}", t, k);
    if let Ok(Some(val)) = bucket.get(tk) {
        val
    } else {
        null
    }
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
    let db = match DB.write() {
        Ok(d) => d,
        Err(e) => {
            warn!("failed get read lock {}", e);
            return null;
        }
    };

    let bucket = match db.bucket::<String, String>(Some(VALUES)) {
        Ok(b) => b,
        Err(e) => {
            warn!("failed get values bucket {}", e);
            return null;
        }
    };

    let tk = format!("{}-{}", token, key);
    if let Ok(_) = bucket.set(tk, body) {
        return key;
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