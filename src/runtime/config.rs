extern crate config;

use config::File;
use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::path::PathBuf;
use std::str::FromStr;

static CONFIG: Lazy<HashMap<String, String>> = Lazy::new(|| {
    let mut settings = config::Config::new();

    // user config
    if let Some(mut path) = dirs::config_dir() {
        path.push("futuresdr");
        path.push("config");
        path.set_extension("toml");

        if path.as_path().exists() {
            if let Err(_) = settings.merge(File::from(path)) {
                panic!("user config broken");
            }
        }
    }

    // project config
    let mut path = PathBuf::new();
    path.push("config");
    path.set_extension("toml");

    if path.as_path().exists() {
        if let Err(_) = settings.merge(File::from(path)) {
            panic!("project config broken");
        }
    }

    // env config
    if let Err(_) = settings.merge(config::Environment::with_prefix("futuresdr")) {
        panic!("env config broken");
    }

    // start from default config
    let mut c = default_config();

    if let Ok(config) = settings.try_into::<HashMap<String, String>>() {
        for (k, v) in config.iter() {
            c.insert(k.clone(), v.clone());
        }
    }
    c
});

fn default_config() -> HashMap<String, String> {
    let mut h = HashMap::new();
    h.insert("buffer_size".to_string(), "32768".to_string());
    h.insert("queue_size".to_string(), "8192".to_string());
    h
}

pub fn get<T: FromStr>(name: &str) -> Option<T> {
    CONFIG.get(name).and_then(|v| v.parse().ok())
}

pub fn get_or_default<T: FromStr>(name: &str, default: T) -> T {
    get(name).unwrap_or(default)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn config() {
        let a = get::<String>("buffer_size");
        assert!(a.is_some());
        assert_eq!(a.unwrap(), default_config()["buffer_size"]);

        let a = get::<usize>("buffer_size");
        assert!(a.is_some());
        assert_eq!(
            a.unwrap(),
            default_config()["buffer_size"].parse::<usize>().unwrap()
        );

        let a = get_or_default::<usize>("foo", 123);
        assert_eq!(a, 123);
    }
}
