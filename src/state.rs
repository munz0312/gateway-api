use reqwest::Client;
use crate::config::get_routes;

#[derive(Clone)]
pub struct AppState {
    pub client: Client,
    pub routes: Vec<Route>,
}

#[derive(Clone)]
pub struct Route {
    pub path: String,
    pub backend_url: String,
}

impl AppState {

    pub fn new() -> Self {
    let client = Client::builder()
        .use_rustls_tls()
        .build()
        .unwrap();
    let routes = get_routes();
    Self { client, routes }
    }

}