mod weather;
mod config;
mod wallpaper;
mod worker;

use std::{env, error::Error, fs::create_dir, io::{ErrorKind, Read}, path::Path};

use reqwest::{blocking::Client, header::{HeaderMap, HeaderValue}};
use serde::{Serialize, Deserialize};
pub use config::Config;
pub use weather::get_weather;

pub use worker::{Worker, Message, MetaMessage};

pub use crate::config::DownloadQuality;
pub use crate::wallpaper::set_wallpaper::set_wallpaper;

const API_BASE_URL: &str = "https://api.unsplash.com";
pub const DEFAULT_CONFIG_PATH: &str = "./config.json";
pub const DEFAULT_DOWNLOAD_PATH: &str = "download";
const MAXIMUM_PER_PAGE: i32 = 100;

#[derive(Serialize, Deserialize, Debug)]
pub struct Urls {
    pub raw: String,
    pub full: String,
    pub regular: String,
    pub small: String,
    pub thumb: String
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SearchResult {
    pub id: String,
    pub urls: Urls
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SearchResults {
    pub total: usize,
    pub results: Vec<SearchResult>
}

pub fn make_unsplash_client(config: &Config) -> Result<Client, Box<dyn Error>> {
    let mut default_headers = HeaderMap::new();
    default_headers.append("Authorization", HeaderValue::from_str(
        &format!("Client-ID {}", match &config.unsplash_access_key {
            Some(key) => key.clone(),
            None => {
                env::var("AWC_UNSPLASH_KEY")?
            }
        }))?);
    let client = Client::builder()
        .default_headers(default_headers)
        .build()?;
    Ok(client)
}

pub fn search_photos(client: &Client, query: &str) -> Result<SearchResults, Box<dyn Error>> {
    let mut response = client.get(format!("{}/search/photos?query={}&per_page={}", 
        API_BASE_URL,
        query,
        MAXIMUM_PER_PAGE)).send()?;
    let mut data = String::new();
    response.read_to_string(&mut data)?;
    let data: SearchResults = serde_json::from_str(&data)?;
    Ok(data)
}

pub fn download_photo(client: &Client, photo: &SearchResult, quality: DownloadQuality) -> Result<String, Box<dyn Error>> {
    let save_path = format!("{}/{}.jpg", DEFAULT_DOWNLOAD_PATH, photo.id); 
    if Path::new(&save_path).exists() {
        return Ok(save_path);
    }
    let url = match quality {
        DownloadQuality::Raw => &photo.urls.raw,
        DownloadQuality::Full => &photo.urls.full,
        DownloadQuality::Regular => &photo.urls.regular,
        DownloadQuality::Small => &photo.urls.small,
        DownloadQuality::Thumb => &photo.urls.thumb
    };
    let mut response = client.get(url).send()?;
    if let Err(e) = create_dir(DEFAULT_DOWNLOAD_PATH) {
        if e.kind() != ErrorKind::AlreadyExists {
            return Err(Box::new(e));
        }
    }
    let mut data = Vec::new();
    response.read_to_end(&mut data)?;

    std::fs::write(&save_path, data)?;
    Ok(save_path)
}

pub struct Hour(pub u32);

impl ToString for Hour {
    fn to_string(&self) -> String {
        match self.0 {
            1 | 2 | 3 => {
                String::from("midnight")
            },
            4 | 5 => {
                String::from("twilight")
            },
            6 | 7 => {
                String::from("sunrise")
            },
            8 | 9 => {
                String::from("morning")
            },
            10 | 11 => {
                String::from("day")
            },
            12 | 13 => {
                String::from("noon")
            },
            14 | 15 | 16 => {
                String::from("afternoon")
            },
            17 | 18 => {
                String::from("sunset")
            },
            19 | 20 => {
                String::from("evening")
            },
            21 | 22 => {
                String::from("night")
            },
            23 | 0 => {
                String::from("late night")
            },
            _ => panic!("Unreachable")
        }
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use chrono::{Local, Timelike};

    use super::*;

    #[test]
    fn test_client() {
        assert!(make_unsplash_client(&Config::from_path(DEFAULT_CONFIG_PATH).unwrap()).is_ok());
    }

    #[test]
    fn test_search() {
        let client = make_unsplash_client(&Config::from_path(DEFAULT_CONFIG_PATH).unwrap()).unwrap();
        assert!(search_photos(&client, "noon").is_ok());
    }

    #[test]
    #[ignore]
    fn test_download() {
        let fake_result = SearchResult {
            id: String::from("6GHNuQAVC8Y"),
            urls: Urls {
                raw: String::new(),
                full: String::from("https://images.unsplash.com/photo-1616762897553-c3a04bcf795d?ixid=MnwyMjk2MjV8MHwxfHNlYXJjaHwxfHxub29ufGVufDB8fHx8MTYyMDU2NzEzNA\\u0026ixlib=rb-1.2.1"),
                regular: String::new(),
                small: String::new(),
                thumb: String::new()
            }
        };
        let path = download_photo(&make_unsplash_client(&Config::from_path(DEFAULT_CONFIG_PATH).unwrap()).unwrap(), &fake_result, DownloadQuality::Full).unwrap();
        assert!(Path::new(&path).exists());
    }

    #[test]
    fn test_generate_config() {
        match std::fs::remove_file("test.json") {
            Ok(_) => {},
            Err(e) => {
                if e.kind() != ErrorKind::NotFound {
                    panic!("{}", e);
                }
            }
        }
        let config = Config::from_path("test.json").unwrap();
        assert_eq!(config.repeat_secs, 1);
        match config.quality {
            DownloadQuality::Full => {},
            _ => panic!("Incorrect quality")
        }
        let config = Config::from_path("test.json").unwrap();
        assert_eq!(config.repeat_secs, 1);
        match config.quality {
            DownloadQuality::Full => {},
            _ => panic!("Incorrect quality")
        }
        match std::fs::remove_file("test.json") {
            Ok(_) => {},
            Err(e) => {
                if e.kind() != ErrorKind::NotFound {
                    panic!("{}", e);
                }
            }
        }
    }

    #[test]
    #[ignore = "It might not be 23:00 now"]
    fn test_now() {
        assert_eq!(Hour(Local::now().hour()).to_string(), "late night");
    }
}
