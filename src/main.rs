use std::{cell::{Ref, RefCell}, rc::Rc};

use awc::{Config, DownloadQuality, DEFAULT_CONFIG_PATH, Message, MetaMessage, Worker, State};
use serde::{Serialize, Deserialize};
use web_view::Content;

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "cmd", rename_all = "camelCase")]
enum Cmd {
    Init,
    Start,
    Stop,
    Lucky,
    UpdateConfig {
        config: Config
    },
    UpdateState
}

fn main() {
    let config = Rc::new(RefCell::new(match Config::from_path(DEFAULT_CONFIG_PATH) {
        Ok(x) => x,
        Err(e) => panic!("Couldn't load or create config: {}", e)
    }));
    let worker = Worker::new();
    let html = include_str!("index.html");
    web_view::builder()
        .title("Automagic Wallpaper Changer")
        .content(Content::Html(html))
        .size(350, 590)
        .resizable(false)
        .debug(true)
        .user_data(config.clone())
        .invoke_handler(|web_view, arg| {
            let cmd: Cmd = serde_json::from_str(arg).unwrap();

            match cmd {
                Cmd::Init => {
                    let config: Rc<RefCell<Config>> = config.clone();
                    let config: Ref<Config> = config.borrow();
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
                    println!("Trying to start worker...");
                    worker.meta_send(MetaMessage::Start);
                },
                Cmd::Stop => {
                    println!("Trying to stop worker...");
                    worker.send(Message::Stop);
                },
                Cmd::Lucky => {
                    worker.send(Message::Redownload);
                },
                Cmd::UpdateConfig { config: new_config } => {
                    let mut config = config.borrow_mut();
                    *config = new_config;
                    match config.save() {
                        Ok(_) => {
                            let state = worker.state.lock().unwrap();
                            match *state {
                                State::Idle | State::Running => {
                                    worker.send(Message::Stop);
                                },
                                State::Stopped => {}
                            }
                            drop(state);
                            worker.meta_send(MetaMessage::Start);
                            web_view.eval("setTimeout(() => { save.classList.remove(\"disabled\"); }, 3000)").unwrap();
                        }
                        Err(_) => {
                            web_view.eval("save.innerHTML = \"Failed\"").unwrap();
                        }
                    }
                },
                Cmd::UpdateState => {
                    let state = worker.state.lock().unwrap();
                    match *state {
                        State::Idle | State::Running => {
                            web_view.eval("render(true)").unwrap();
                        },
                        State::Stopped => {
                            web_view.eval("render(false)").unwrap();
                        }
                    }
                    drop(state);
                }
            }
            Ok(())
        })
        .run()
        .unwrap();
}
