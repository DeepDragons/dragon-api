extern crate reqwest;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;

mod web_api;
use web_api::{routes::get_dragons, routes::get_dragon_by_id, routes::get_from_battle, routes::get_from_breed, routes::get_from_market};
mod state;
use state::reciver::create_app_state;
use tide::http::headers::HeaderValue;

pub const DEFAULT_API_URL: &str = "127.0.0.1:8083";

#[tokio::main]
async fn main() -> tide::Result<()> {
    dotenv::dotenv().ok();

    let app_state = create_app_state().await;
    let api_url = match std::env::var("API_URL") {
        Ok(val) => val,
        Err(_) => String::from(DEFAULT_API_URL),
    };
    println!("Dragons backend is starting on {}", api_url);
    let mut app = tide::with_state(app_state);
    #[cfg(debug_assertions)]
    {
        println!("Debug mode, CORS allow \"*\"");
        let cors_debug = tide::security::CorsMiddleware::new()
            .allow_methods("GET".parse::<HeaderValue>().unwrap())
            .allow_origin(tide::security::Origin::from("*"))
            .allow_credentials(false);
        app.with(cors_debug);
    }
    app.at("/api/v1/dragons").get(get_dragons);
    app.at("/api/v1/dragons/:id").get(get_dragon_by_id);
    app.at("/api/v1/market").get(get_from_market);
    app.at("/api/v1/battle").get(get_from_battle);
    app.at("/api/v1/breed").get(get_from_breed);
    app.listen(api_url).await?;
    Ok(())
}

