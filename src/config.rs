use easy_toml_config::*;

use std::collections::BTreeMap;
use std::fs::File;

fn config_file() -> String {
    format!("{}.toml", super::PLUGIN_NAME)
}

lazy_static! {
    pub static ref CONFIG: Config = {
        read_config(&config_file(), config_template())
    };
}

/// Reads the config file
fn read_config(config_file: &str, config: Config) -> Config {
    use std::io::Read;
    let mut config_file = init(config_file, config);

    let mut data = String::new();
    config_file.read_to_string(&mut data);
    error_handler(toml::from_str(&data))
}

/// Main struct for config
#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct Config {
    pub enable: bool,
    pub channels: Option<BTreeMap<String, Options>>,
}

/// This struct is meant for special/extra settings for the individual channels,
/// like exceptions for users that are allowed to post in channels or
/// that the channel also don't allow threads
#[derive(Deserialize,Serialize,Clone,Debug)]
pub struct Options {
}

/// The setting/config are saved
impl WriteConfig for Config {
    fn write(&self) {
        use std::io::Write;

        let path_config_file = path_config_file(&config_file());

        let mut config_file = File::create(&path_config_file).expect(&format!("Failed at creating a template config file '{}'", &path_config_file.to_str().unwrap()));

        let toml = toml::to_string(self).unwrap();
        config_file.write_all(toml.as_bytes()).expect(&format!("Failed to create a config file"));

        println!("Edit the config file '{}'", &path_config_file.to_str().unwrap());
    }
}

/// Create a example/default configuration
fn config_template() -> Config {
    Config {
        enable: false,
        channels: Some({
            let mut channels = BTreeMap::new();
            channels.insert("#test".to_string(), Options { });
            channels
        }),
    }
}
