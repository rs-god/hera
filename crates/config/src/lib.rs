use serde_yaml::{self, Value};
use std::fs::File;
use std::io::{Error, Read};
use std::path::{Path, PathBuf};

// ConfigTrait define config trait
pub trait ConfigTrait {
    fn load(&mut self) -> Result<(), Error>;
    fn sections(&self) -> Value;
    fn content(&self) -> &str;
}

// Config impl config trait
pub struct Config {
    path: PathBuf,
    sections: String,
}

impl Config {
    // 创建一个Config实例
    pub fn new<P: AsRef<Path>>(path: P) -> Self {
        let p = PathBuf::from(path.as_ref());
        let s = Config {
            path: p,
            sections: String::new(),
        };
        s
    }
}

impl ConfigTrait for Config {
    fn load(&mut self) -> Result<(), Error> {
        File::open(&self.path)
            .unwrap()
            .read_to_string(&mut self.sections)?;
        Ok(())
    }

    fn sections(&self) -> Value {
        let val = serde_yaml::from_str(&self.sections).unwrap();
        val
    }

    fn content(&self) -> &str {
        self.sections.as_str()
    }
}

#[test]
fn test_config() {
    use serde::{Deserialize, Serialize};

    #[derive(Debug, PartialEq, Serialize, Deserialize, Default)]
    struct AppConfig {
        app_debug: bool,
        app_name: String,
        app_port: i32,
    }

    let mut c = Config::new("test.yaml");
    c.load().expect("read file failed");

    // read config to struct
    let s: AppConfig = serde_yaml::from_str(c.content()).unwrap();
    println!("{:?}", s);

    // read config from serde Value
    let s: AppConfig = serde_yaml::from_value(c.sections()).unwrap();
    println!("{:?}", s);
}
