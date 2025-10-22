use crate::state::Route;

pub fn match_route<'a>(routes: &'a Vec<Route>, path: &str) -> Option<&'a Route> {
    routes
    .iter()
    .find(|route| path.starts_with(&route.path))
}