use std::{process, thread, time::Duration};

use awc::{Config, DEFAULT_CONFIG_PATH, Hour, download_photo, get_weather, make_unsplash_client, search_photos, set_wallpaper};
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
    let client = match make_unsplash_client(&config) {
        Ok(client) => client,
        Err(_) => {
            eprintln!("\
Looks like this is the first time you use AWC.
You need to do a few things first - otherwise AWC won't run.
A `config.json` has been generated under the cwd, please modify
it to your heart's content, especially the `unsplash_access_key`.
I can't download wallpapers without it.

You can apply for one here: https://unsplash.com/join

If you also want the wallpaper to (poorly) resemble your current
weather, you can apply an OpenWeather API here: openweathermap.org
you will also need to change the city you are in.
Otherwise, you can just leave it as it is.
WARNING: weather messes up with the query term, and you might get
wallpapers in the wrong time.

`repeat_secs` is the interval before I download another wallpaper -
it is one hour by default.

The quality is splitted into 5 levels, as the Unsplash API states:
Raw, Full, Regular, Small and Thumb.

Run the program again after you've updated the configs accordingly.

Have a lot of fun...");
            process::exit(1);
        }
    };
    let mut attempts = 0;
    loop {
        let now = Local::now();
        let mut query = Hour(now.hour()).to_string();
        match &config.openweather_access_key {
            Some(x) => match get_weather(x, &config.city_weather) {
                Ok(x) => {
                    query.push(' ');
                    query.push_str(&x);
                },
                Err(e) => {
                    eprintln!("Failed to get weather information: {} Skipping...", e);
                }
            },
            None => {}
        };
        
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
        println!("New photo downloaded at: {}. Setting wallpaper...", path);
        
        match set_wallpaper(&path) {
            Ok(_) => {}
            Err(e) => {
                eprintln!("Failed to set wallpaper: {}. Stopping...", e);
                process::exit(1);
            }
        }

        thread::sleep(Duration::from_secs(config.repeat_secs));
    }
}
