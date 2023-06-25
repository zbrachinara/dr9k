use std::sync::OnceLock;

use serde::Deserialize;
use twilight_model::id::{marker::ApplicationMarker, Id};

#[derive(Deserialize)]
#[serde(from = "ConfigRaw")]
pub struct Config {
    pub discord_token: String,
    pub application_id: Id<ApplicationMarker>,
}

#[derive(Deserialize)]
pub struct ConfigRaw {
    discord_token: String,
    application_id: u64,
}

impl From<ConfigRaw> for Config {
    fn from(value: ConfigRaw) -> Self {
        Config {
            discord_token: value.discord_token,
            application_id: Id::new(value.application_id),
        }
    }
}

static CONFIG: OnceLock<Config> = OnceLock::new();
pub fn config() -> &'static Config {
    CONFIG.get_or_init(|| {
        let config_str =
            std::fs::read_to_string("./dr9k.toml").expect("`dr9k.toml` could not be opened");
        toml::from_str(&config_str).expect("`dr9k.toml` was incorrectly configured")
    })
}
