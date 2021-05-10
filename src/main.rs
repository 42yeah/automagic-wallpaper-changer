use std::thread::sleep;

use awc::{Config, DownloadQuality, DEFAULT_CONFIG_PATH, Message, MetaMessage, Worker};
use serde::{Serialize, Deserialize};
use web_view::Content;

#[derive(Serialize, Deserialize)]
#[serde(tag = "cmd", rename_all = "camelCase")]
enum Cmd {
    Init,
    Start,
    Stop,
    Lucky
}

fn main() {
    let mut config = match Config::from_path(DEFAULT_CONFIG_PATH) {
        Ok(x) => x,
        Err(e) => panic!("Couldn't load or create config: {}", e)
    };
    let worker = Worker::new();
    let html = include_str!("index.html");
    web_view::builder()
        .title("Automagic Wallpaper Changer")
        .content(Content::Html(html))
        .size(350, 590)
        .resizable(false)
        .debug(true)
        .user_data(&config)
        .invoke_handler(|web_view, arg| {
            let cmd: Cmd = serde_json::from_str(arg).unwrap();

            match cmd {
                Cmd::Init => {
                    match web_view.eval(&format!("
                    document.querySelector('#repeat-secs').value = {};
                    document.querySelector('#wallpaper-interval').value = {};
                    document.querySelector('#unsplash-access-key').value = '{}';
                    document.querySelector('#openweather-access-key').value = '{}';
                    document.querySelector('#city').value = '{}'
                    document.querySelector('#quality').value = '{}';",
                    config.repeat_secs,
                    config.update_interval,
                    match &config.unsplash_access_key {
                        Some(x) => x,
                        None => ""
                    },
                    match &config.openweather_access_key {
                        Some(x) => x,
                        None => ""
                    },
                    &config.city_weather,
                    match &config.quality {
                        DownloadQuality::Raw => "Raw",
                        DownloadQuality::Full => "Full",
                        DownloadQuality::Regular => "Regular",
                        DownloadQuality::Small => "Small",
                        DownloadQuality::Thumb => "Thumb"
                    })) {
                        Ok(_) => {}
                        Err(e) => {
                            eprintln!("There are some errors while filling the blanks: {}. The HTML config form will not be filled.", e);
                        }
                    }
                },
                Cmd::Start => {
                    worker.meta_send(MetaMessage::Start);
                },
                Cmd::Stop => {
                    worker.send(Message::Stop);
                },
                Cmd::Lucky => {
                    worker.send(Message::Redownload);
                }
            }
            Ok(())
        })
        .run()
        .unwrap();
}
