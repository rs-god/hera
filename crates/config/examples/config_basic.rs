use config::{Config, ConfigTrait};
use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Debug, PartialEq, Serialize, Deserialize, Default)]
struct AppConfig {
    // 通过 #[serde(default)] 指定默认值
    #[serde(default)]
    app_debug: bool,

    #[serde(default)]
    app_name: String,

    #[serde(default = "default_app_port")]
    app_port: i32,
    graceful_wait_time: Duration,
}

fn default_app_port() -> i32 {
    8080
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Default)]
struct AppConf {
    graceful_wait_time: Duration,
}

// cargo run --example config_basic
fn main() {
    let mut c = Config::new("test.yaml");
    c.load().expect("read file failed");

    // read config to struct
    let s: AppConfig = serde_yaml::from_str(c.content()).unwrap();
    println!("{:?}", s);

    // read config from serde Value
    // let s: AppConfig = serde_yaml::from_value(c.sections()).unwrap();
    // println!("{:?}", s);

    let a = AppConf {
        graceful_wait_time: Duration::from_secs(10),
    };
    let s = serde_yaml::to_string(&a).unwrap();
    println!("app conf:\n{}", s);
}

/*
output:
AppConfig { app_debug: true, app_name: "test", app_port: 1336 }
AppConfig { app_debug: true, app_name: "test", app_port: 1336 }
 */
