use config::{Config, ConfigTrait};
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Serialize, Deserialize, Default)]
struct AppConfig {
    app_debug: bool,
    app_name: String,
    app_port: i32,
}

// cargo run --example config_basic
fn main() {
    let mut c = Config::new("test.yaml");
    c.load().expect("read file failed");

    // read config to struct
    let s: AppConfig = serde_yaml::from_str(c.content()).unwrap();
    println!("{:?}", s);

    // read config from serde Value
    let s: AppConfig = serde_yaml::from_value(c.sections()).unwrap();
    println!("{:?}", s);
}

/*
output:
AppConfig { app_debug: true, app_name: "test", app_port: 1336 }
AppConfig { app_debug: true, app_name: "test", app_port: 1336 }
 */
