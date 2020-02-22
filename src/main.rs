extern crate pretty_env_logger;

mod lib;
use lib::{get_value, post_value, random_token, write_token};

use log::info;
use warp::Filter;

/*
TODO: /stats
*/

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

    let get = warp::path!(String / String).and(warp::get()).map(get_value);

    let post = warp::path!(String / String)
        .and(warp::post())
        .and(warp::body::content_length_limit(1024 * 4)) // limit to 4KB body
        .and(warp::body::bytes())
        .map(post_value);

    // TODO: /stats
    let routes = new_token.or(get).or(post);

    info!("starting server at {}", 8080);
    warp::serve(routes).run(([0, 0, 0, 0], 8080)).await;
}
