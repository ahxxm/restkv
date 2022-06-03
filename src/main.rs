extern crate pretty_env_logger;

mod lib;
use lib::{list_keys, get_value, post_value, random_token, write_token, stats};

use log::info;
use warp::Filter;

#[tokio::main]
async fn main() {
    pretty_env_logger::init();
    // TODO: limit global rate?
    let new_token = warp::path!("new").and(warp::post()).map(|| {
        let token = random_token();
        let written = write_token(&token);
        if written {
            token
        } else {
            "".to_string()
        }
    });

    let keys = warp::path!("keys" / String).and(warp::get()).map(list_keys);

    let get = warp::path!(String / String).and(warp::get()).map(get_value);

    let post = warp::path!(String / String)
        .and(warp::post())
        .and(warp::body::content_length_limit(1024 * 4)) // limit to 4KB body
        .and(warp::body::bytes())
        .map(post_value);

    let stat = warp::path!("stats")
        .and(warp::get())
        .map(stats);

    let homepage = warp::any().map(||"https://github.com/ahxxm/restkv");
    let routes = new_token.or(keys).or(get).or(post).or(stat).or(homepage);

    info!("starting server at {}", 28080);
    warp::serve(routes).run(([0, 0, 0, 0], 28080)).await;
}
