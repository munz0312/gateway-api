use crate::state::Route;
use std::fs;
use serde_json::Value;

pub fn get_routes() -> Vec<Route> {
    let config = fs::read_to_string("config.json").expect("Couldn't read config file");
    let config: Value = serde_json::from_str(config.as_str()).unwrap();
    let routes = config["routes"].as_array().unwrap();

    routes
        .iter()
        .map(|v| Route {
            path: v["path"].as_str().unwrap_or_default().to_string(),
            backend_url: v["backend_url"].as_str().unwrap_or_default().to_string(),
        })
        .collect()
}