use std::{error::Error, path::Path};
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum DownloadQuality {
    Raw,
    Full,
    Regular,
    Small,
    Thumb
}

#[derive(Serialize, Deserialize)]
pub struct Config {
    pub repeat_secs: u64,
    pub update_interval: u64,
    pub quality: DownloadQuality,
    pub unsplash_access_key: Option<String>,
    pub openweather_access_key: Option<String>,
    pub city_weather: String,
    pub disable_cache: bool
}

impl Config {
    pub fn from_path(path: &str) -> Result<Config, Box<dyn Error>> {
        let path = Path::new(path);
        if path.exists() {
            return Ok(serde_json::from_str(&std::fs::read_to_string(&path)?)?)
        }
        let config = Config {
            repeat_secs: 1,
            update_interval: 3600,
            quality: DownloadQuality::Full,
            unsplash_access_key: None,
            openweather_access_key: None,
            city_weather: String::from("Dublin"),
            disable_cache: true
        };
        let config_str = serde_json::to_string(&config)?;
        std::fs::write(path, config_str)?;
        Ok(config)
    }
}