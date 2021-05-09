use std::{process, thread, time::Duration};

use awc::{Config, DEFAULT_CONFIG_PATH, Hour, download_photo, make_client, search_photos};
use chrono::{Local, Timelike};
use rand::Rng;

const MAXIMUM_ATTEMPTS: i32 = 5;
const WAIT_SECS: u64 = 60;

fn main() {
    let config = match Config::from_path(DEFAULT_CONFIG_PATH) {
        Ok(config) => config,
        Err(e) => {
            eprintln!("Failed to read config: {}", e);
            process::exit(1);
        }
    };
    let client = match make_client() {
        Ok(client) => client,
        Err(e) => {
            eprintln!("Failed to make client: {}", e);
            process::exit(1);
        }
    };
    let mut attempts = 0;
    loop {
        let now = Local::now();
        let query = Hour(now.hour()).to_string();
        println!("Trying to search from unsplash with: {}", query);
        
        let results = match search_photos(&client, &query) {
            Ok(results) => {
                results
            },
            Err(e) => {
                attempts += 1;
                if attempts >= MAXIMUM_ATTEMPTS {
                    eprintln!("Failed to get photo list: {}. Too many retries. Stopping...", e);
                    break;
                } else {
                    eprintln!("Failed to get photo list: {}, Trying again in {} seconds...", e, WAIT_SECS);
                    thread::sleep(Duration::from_secs(WAIT_SECS));
                }
                continue;
            }
        };
        let choice = rand::thread_rng().gen_range(0..results.results.len());
        let choice = &results.results[choice];
        
        let path = match download_photo(&client, choice, config.quality.clone()) {
            Ok(path) => path,
            Err(e) => {
                attempts += 1;
                if attempts >= MAXIMUM_ATTEMPTS {
                    eprintln!("Download failed: {}. Too many retries. Stopping...", e);
                    break;
                } else {
                    eprintln!("Download failed: {}. Trying again in {} seconds...", e, WAIT_SECS);
                    thread::sleep(Duration::from_secs(WAIT_SECS));
                }
                continue;
            }
        };

        attempts = 0;
        println!("New photo downloaded at: {}", path);

        thread::sleep(Duration::from_secs(config.repeat_secs));
    }
}
