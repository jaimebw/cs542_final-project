use rocket::figment::value::Value as FigmentValue;
use rocket::http::uri::Host;
use rocket::{Build, Rocket, Route};
use serde_json::Value;
use std::collections::HashMap;
use std::fmt::Write;
use tera::Function;

#[derive(Clone, Debug)]
pub struct TemplateUrlLoader {
    server_name: Option<Host<'static>>,
    default_scheme: String,
    routes: Vec<Route>,
}

impl From<&Rocket<Build>> for TemplateUrlLoader {
    fn from(rocket: &Rocket<Build>) -> Self {
        let figment = rocket.figment();

        let server_name = figment.find_value("address").ok().and_then(|x| {
            if let FigmentValue::String(tag, value) = x {
                if !tag.is_default() {
                    return Host::parse_owned(value).ok();
                }
            }
            None
        });

        let default_scheme = match figment.find_value("tls.key") {
            Ok(_) => "https".to_string(),
            Err(_) => "http".to_string(),
        };

        TemplateUrlLoader {
            server_name,
            default_scheme,
            routes: rocket.routes().cloned().collect(),
        }
    }
}

impl TemplateUrlLoader {
    fn find_route(&self, endpoint: &str, args: &HashMap<String, Value>) -> Option<&Route> {
        let method = args.get("_method");

        self.routes
            .iter()
            .filter(|route| {
                if let Some(requested_method) = method {
                    if requested_method != route.method.as_str() {
                        return false;
                    }
                }

                match &route.name {
                    Some(name) => name == endpoint,
                    None => false,
                }
            })
            .fold(None, |a, b| match a {
                Some(x) if x.rank > b.rank => Some(x),
                _ => Some(b),
            })
    }

    fn write_path_parameters(
        &self,
        buffer: &mut String,
        route: &Route,
        args: &HashMap<String, Value>,
    ) -> tera::Result<()> {
        let mut path_str = route.uri.path();
        while !path_str.is_empty() {
            let parts = path_str.split_once('<').and_then(|(prefix, remaining)| {
                let (key, suffix) = remaining.split_once('>')?;
                Some((prefix, key, suffix))
            });

            if let Some((prefix, mut key, suffix)) = parts {
                buffer.push_str(prefix);
                path_str = suffix;

                if let Some(short_name) = key.strip_suffix("..") {
                    key = short_name;
                }

                match args.get(key) {
                    Some(Value::String(s)) => buffer.push_str(s),
                    Some(x) => {
                        // Attempt to shove whatever value we have into the URI as JSON. If we push
                        // something weird, it should error when we pass it to the url crate.
                        if write!(buffer, "{}", x).is_err() {
                            unreachable!("Writing to a string will never fail")
                        }
                    }
                    None => {
                        let msg = format!(
                            "Unable to find required parameter {:?} for route {:?}",
                            key, route.name
                        );
                        return Err(tera::Error::msg(msg));
                    }
                }
            } else {
                buffer.push_str(path_str);
                break;
            }
        }

        Ok(())
    }

    fn write_query_parameters(
        &self,
        buffer: &mut String,
        route: &Route,
        args: &HashMap<String, Value>,
    ) {
        let mut query_str = match route.uri.query() {
            Some(x) => x,
            None => return,
        };

        buffer.push('?');
        while !query_str.is_empty() {
            let parts = query_str.split_once('<').and_then(|(prefix, remaining)| {
                let (key, suffix) = remaining.split_once('>')?;
                Some((prefix, key, suffix))
            });

            if let Some((prefix, key, suffix)) = parts {
                buffer.push_str(prefix);
                query_str = suffix;

                match args.get(key) {
                    Some(Value::Null) | Some(Value::Bool(false)) | None => {}
                    Some(Value::String(s)) => {
                        buffer.push_str(key);
                        buffer.push('=');
                        buffer.push_str(s)
                    }
                    Some(Value::Bool(true)) => buffer.push_str(key),
                    Some(x) => {
                        // Attempt to shove whatever value we have into the URI as JSON. If we push
                        // something weird, it should error when we pass it to the url crate.
                        if write!(buffer, "{}={}", key, x).is_err() {
                            unreachable!("Writing to a string will never fail")
                        }
                    }
                }
            } else {
                buffer.push_str(query_str);
                break;
            }
        }
    }

    fn clean_query_parameters(buffer: &mut String) {
        let is_query_separator = |c: char| c == '&' || c == '?';

        // Dedup query separators
        let mut last_was_separator = false;
        buffer.retain(|next| {
            if last_was_separator && is_query_separator(next) {
                return false;
            }

            last_was_separator = is_query_separator(next);
            true
        });

        // Remove any trailing query separators. This isn't required, but it makes the urls cleaner
        // and helps for consistent unit testing.
        while let Some(last) = buffer.pop() {
            if !is_query_separator(last) {
                buffer.push(last);
                break;
            }
        }
    }
}

impl Function for TemplateUrlLoader {
    fn call(&self, args: &HashMap<String, Value>) -> tera::Result<Value> {
        let endpoint = match args.get("endpoint") {
            Some(Value::String(v)) => v,
            Some(x) => {
                let msg = format!("endpoint argument must be string. Received value: {}", x);
                return Err(tera::Error::msg(msg));
            }
            None => {
                return Err(tera::Error::msg(
                    "url_for was called without endpoint argument",
                ))
            }
        };

        let external = match args.get("_external") {
            Some(Value::Bool(x)) => *x,
            None => self.server_name.is_some(),
            Some(x) => {
                let msg = format!(
                    "{}: _external argument must be boolean. Received value: {}",
                    endpoint, x
                );
                return Err(tera::Error::msg(msg));
            }
        };

        let mut buffer = String::new();

        if external {
            let scheme = match args.get("_scheme") {
                Some(Value::String(x)) => x,
                None => &self.default_scheme,
                Some(x) => {
                    let msg = format!(
                        "{}: _scheme argument must be string. Received value: {}",
                        endpoint, x
                    );
                    return Err(tera::Error::msg(msg));
                }
            };

            let server_name = match self.server_name.as_ref() {
                Some(x) => x,
                None => {
                    return Err(tera::Error::msg(
                        "Unable to write external url without address specified in Rocket.toml",
                    ))
                }
            };

            buffer.push_str(scheme);
            buffer.push_str("://");

            if write!(&mut buffer, "{}", server_name).is_err() {
                unreachable!("IO errors will not occur when writing to String")
            }
        }

        match self.find_route(endpoint, args) {
            Some(route) => {
                self.write_path_parameters(&mut buffer, route, args)?;
                self.write_query_parameters(&mut buffer, route, args);
            }
            None if endpoint.starts_with('.') => {
                // We are using a relative path so substitute it for the URI
                if endpoint != "." {
                    buffer.push_str(endpoint);
                }
            }
            None => {
                let msg = format!("Unable to find route for name {:?}", endpoint);
                return Err(tera::Error::msg(msg));
            }
        }

        match args.get("_anchor") {
            Some(Value::String(x)) => {
                buffer.push('#');
                buffer.push_str(x);
            }
            Some(x) => {
                let msg = format!(
                    "{}: _anchor argument must be string. Received value: {}",
                    endpoint, x
                );
                return Err(tera::Error::msg(msg));
            }
            None => {}
        }

        Self::clean_query_parameters(&mut buffer);
        Ok(Value::String(buffer))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rocket::figment::Figment;
    use rocket::{get, post, routes};

    #[get("/abc/<x>/foo?<y>")]
    fn test_route(x: &str, y: u32) -> String {
        format!("x: {:?}, y: {}", x, y)
    }

    #[post("/abc/<x>/foo?<y>&<z>")]
    pub fn foo_post(x: &str, y: Option<u32>, z: u32) -> String {
        format!("x: {:?}, y: {:?}, z: {}", x, y, z)
    }

    #[get("/foo")]
    fn foo_get() -> &'static str {
        ""
    }

    fn test_routes(external: bool) -> Rocket<Build> {
        // Create empty config so tests are not influenced by project configuration
        let mut figment = Figment::new();

        if external {
            figment = figment.merge(("address", "example.com"));
        }

        rocket::custom(figment)
            .mount("/", routes![test_route, foo_post])
            .mount("/bar", routes![foo_get])
    }

    #[test]
    pub fn find_route() {
        let rocket_build = test_routes(false);
        let url_loader = TemplateUrlLoader::from(&rocket_build);

        let mut args = HashMap::new();
        args.insert("endpoint".to_string(), Value::String("foo_get".to_string()));

        assert_eq!(
            url_loader.call(&args).unwrap(),
            Value::String("/bar/foo".to_string())
        );
    }

    #[test]
    pub fn attempt_replacement() {
        let rocket_build = test_routes(false);
        let url_loader = TemplateUrlLoader::from(&rocket_build);

        let mut args = HashMap::new();
        args.insert(
            "endpoint".to_string(),
            Value::String("test_route".to_string()),
        );
        args.insert("x".to_string(), Value::String("xyz".to_string()));
        args.insert("y".to_string(), Value::Number(123.into()));

        assert_eq!(
            url_loader.call(&args).unwrap(),
            Value::String("/abc/xyz/foo?y=123".to_string())
        );
    }

    #[test]
    pub fn attempt_replacement_with_optional() {
        let rocket_build = test_routes(false);
        let url_loader = TemplateUrlLoader::from(&rocket_build);

        let mut args = HashMap::new();
        args.insert(
            "endpoint".to_string(),
            Value::String("foo_post".to_string()),
        );
        args.insert("x".to_string(), Value::String("xyz".to_string()));
        args.insert("z".to_string(), Value::Number(123.into()));

        // Check with y not present
        assert_eq!(
            url_loader.call(&args).unwrap(),
            Value::String("/abc/xyz/foo?z=123".to_string())
        );

        // Test will null value (equivalent to None)
        args.insert("y".to_string(), Value::Null);
        assert_eq!(
            url_loader.call(&args).unwrap(),
            Value::String("/abc/xyz/foo?z=123".to_string())
        );

        args.insert("y".to_string(), Value::Number(456.into()));
        assert_eq!(
            url_loader.call(&args).unwrap(),
            Value::String("/abc/xyz/foo?y=456&z=123".to_string())
        );
    }

    #[test]
    pub fn attempt_external() {
        let rocket_build = test_routes(true);
        let url_loader = TemplateUrlLoader::from(&rocket_build);

        let mut args = HashMap::new();
        args.insert("endpoint".to_string(), Value::String("foo_get".to_string()));

        // external enabled by default when address is present
        assert_eq!(
            url_loader.call(&args).unwrap(),
            Value::String("http://example.com/bar/foo".to_string())
        );

        args.insert("_external".to_string(), Value::Bool(true));
        assert_eq!(
            url_loader.call(&args).unwrap(),
            Value::String("http://example.com/bar/foo".to_string())
        );

        args.insert("_external".to_string(), Value::Bool(false));
        assert_eq!(
            url_loader.call(&args).unwrap(),
            Value::String("/bar/foo".to_string())
        );
    }
}
