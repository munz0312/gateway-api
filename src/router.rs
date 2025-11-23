use crate::state::Route;

pub fn match_route<'a>(routes: &'a Vec<Route>, path: &str) -> Option<&'a Route> {
    routes.iter().find(|route| {
        if route.path == "/" {
            return path.starts_with('/');
        }
        if path.starts_with(&route.path) {
            let remaining = &path[route.path.len()..];
            remaining.is_empty() || remaining.starts_with('/') || remaining.starts_with('?')
        } else {
            false
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_routes() -> Vec<Route> {
        vec![
            Route {
                path: "/api".to_string(),
                backend_url: "http://api.example.com".to_string(),
            },
            Route {
                path: "/auth".to_string(),
                backend_url: "http://auth.example.com".to_string(),
            },
            Route {
                path: "/v1/users".to_string(),
                backend_url: "http://users.example.com".to_string(),
            },
        ]
    }

    #[test]
    fn test_exact_path_match() {
        let routes = create_test_routes();
        let result = match_route(&routes, "/api");

        assert!(result.is_some());
        assert_eq!(result.unwrap().path, "/api");
        assert_eq!(result.unwrap().backend_url, "http://api.example.com");
    }

    #[test]
    fn test_prefix_path_match() {
        let routes = create_test_routes();
        let result = match_route(&routes, "/api/users/123");

        assert!(result.is_some());
        assert_eq!(result.unwrap().path, "/api");
    }

    #[test]
    fn test_nested_path_match() {
        let routes = create_test_routes();
        let result = match_route(&routes, "/v1/users/profile");

        assert!(result.is_some());
        assert_eq!(result.unwrap().path, "/v1/users");
        assert_eq!(result.unwrap().backend_url, "http://users.example.com");
    }

    #[test]
    fn test_no_match() {
        let routes = create_test_routes();
        let result = match_route(&routes, "/nonexistent");

        assert!(result.is_none());
    }

    #[test]
    fn test_first_match_wins() {
        let routes = vec![
            Route {
                path: "/api".to_string(),
                backend_url: "http://api1.example.com".to_string(),
            },
            Route {
                path: "/api/users".to_string(),
                backend_url: "http://api2.example.com".to_string(),
            },
        ];

        // Should match the first route since it comes first
        let result = match_route(&routes, "/api/users/123");
        assert!(result.is_some());
        assert_eq!(result.unwrap().backend_url, "http://api1.example.com");
    }

    #[test]
    fn test_empty_routes() {
        let routes = Vec::new();
        let result = match_route(&routes, "/any/path");

        assert!(result.is_none());
    }

    #[test]
    fn test_root_path() {
        let routes = vec![Route {
            path: "/".to_string(),
            backend_url: "http://root.example.com".to_string(),
        }];

        let result = match_route(&routes, "/anything");
        assert!(result.is_some());
        assert_eq!(result.unwrap().path, "/");
    }

    #[test]
    fn test_path_with_query_params() {
        let routes = create_test_routes();
        let result = match_route(&routes, "/api/search?q=test");

        assert!(result.is_some());
        assert_eq!(result.unwrap().path, "/api");
    }

    #[test]
    fn test_similar_paths() {
        let routes = vec![
            Route {
                path: "/app".to_string(),
                backend_url: "http://app.example.com".to_string(),
            },
            Route {
                path: "/application".to_string(),
                backend_url: "http://application.example.com".to_string(),
            },
        ];

        let result = match_route(&routes, "/application/config");
        assert!(result.is_some());
        assert_eq!(result.unwrap().path, "/application");
    }
}

