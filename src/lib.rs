#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate serde_derive;
extern crate easy_toml_config;

#[macro_use]
extern crate log;
extern crate simple_logging;
use log::LevelFilter;
use std::io;

mod config;
mod misc;

extern crate template;
use template::plugin_api_v1::*;
use std::path::PathBuf;

static PLUGIN_NAME: &str = "block_posts";

/// Tells BEST-Bot what version of the api the plugin uses.
/// Note: The plugin will not be loaded if this function is missing.
#[no_mangle]
pub extern "C" fn api_version() -> u32 {
    1
}

/// Creates the plugin object which The BEST-Bot needs
#[no_mangle]
pub extern "C" fn load() -> Box<Plugin> {
    Box::new(Base {
        slack: None,
    })
}

struct Base {
    slack: Option<Slack>,
}

impl Plugin for Base {

    /// Returns the name of the plugin to The BEST-Bot
    fn name(&self) -> &'static str {
        PLUGIN_NAME
    }

    /// This function is called right after the plugin is loaded into BEST-Bot
    fn on_plugin_load(&mut self, slack: Slack, config_folder: PathBuf) {
        easy_toml_config::set_config_dir(config_folder.to_str().unwrap().to_string());
        simple_logging::log_to(io::stdout(), LevelFilter::Info);
        self.slack = Some(slack);
    }

    /// Tells BEST-Bot which events/actions the plugin requires to work
    fn event_subscript(&self) -> Vec<EVENT_SUBSCRIBE> {
        vec![EVENT_SUBSCRIBE::STANDARD_MESSAGE]
    }

    /// Then one of the event the plugin subscript to are triggered, this functions is called and
    /// the information are handed over to the plugin.
    fn event(&self, event: EVENT) {
        if config::CONFIG.enable {
            match event {
                EVENT::STANDARD_MESSAGE(standard_message) => {
                    misc::delete_post_from_channel(standard_message, &self.slack.as_ref().unwrap())
                },
            }
        }
    }
}