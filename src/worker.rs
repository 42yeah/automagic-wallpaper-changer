
use std::{rc::Rc, sync::{self, Arc, Mutex, mpsc::{self, Receiver, Sender}}, thread::{self, JoinHandle}};
use std::{time::{Duration, SystemTime}};
use chrono::{Local, Timelike};
use rand::Rng;

use crate::{Config, DEFAULT_CONFIG_PATH, DEFAULT_DOWNLOAD_PATH, Hour, download_photo, get_weather, make_unsplash_client, search_photos, set_wallpaper};

const MAXIMUM_ATTEMPTS: i32 = 5;
const WAIT_SECS: u64 = 60;

#[derive(Debug)]
pub enum Message {
    Redownload,
    Terminate
}

pub enum MetaMessage {
    Worker(Arc<Mutex<Worker>>),
    Start,
    Quit
}

pub enum State {
    Idle,
    Running,
    Stopped
}

pub struct Worker {
    thread: JoinHandle<()>,
    sender: Sender<Message>,
    meta_sender: Arc<Mutex<Sender<MetaMessage>>>,
    state: State
}

impl Worker {
    pub fn new() -> Arc<Mutex<Worker>> {
        let (tx, rx) = mpsc::channel();
        let (sender, receiver) = mpsc::channel();
        let rx_ptr = Arc::new(Mutex::new(rx));
        let thread = thread::spawn(move || {
            let worker: MetaMessage = rx_ptr.lock().unwrap().recv().unwrap();
            let mut worker = match worker {
                MetaMessage::Worker(worker) => worker,
                _ => panic!("Unreachable: Wrong meta message")
            };

            loop {
                Worker::work(worker.clone(), &receiver);
                match rx_ptr.lock().unwrap().recv().unwrap() {
                    MetaMessage::Worker(new_worker) => { worker = new_worker },
                    MetaMessage::Start => {},
                    MetaMessage::Quit => break
                }
            }
            
        });

        let meta_sender = Arc::new(Mutex::new(tx));
        let worker = Arc::new(Mutex::new(Worker {
            thread,
            sender,
            meta_sender: meta_sender.clone(),
            state: State::Stopped
        }));
        meta_sender.lock().unwrap().send(MetaMessage::Worker(worker.clone())).unwrap();

        worker
    }

    fn work(worker: Arc<Mutex<Worker>>, receiver: &Receiver<Message>) {
        let config = match Config::from_path(DEFAULT_CONFIG_PATH) {
            Ok(config) => config,
            Err(e) => {
                eprintln!("Failed to read config: {}", e);
                return;
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
                return;
            }
        };
        let mut attempts = 0;
        let mut last_instant: Option<SystemTime> = None;
        let mut last_path: Option<String> = None;
    
        if config.disable_cache {
            match std::fs::remove_dir_all(DEFAULT_DOWNLOAD_PATH) {
                Ok(_) => {
                    println!("Image cache directory has been cleaned up");
                }
                Err(_) => {}
            }
        }
    
        loop {
            match receiver.try_recv() {
                Ok(msg) => {
                    match msg {
                        Message::Terminate => return,
                        Message::Redownload => last_instant = None
                    }
                }
                Err(_) => {}
            }

            let now = Local::now();
            let this_instant = SystemTime::now();

            let mut worker_locked = worker.lock().unwrap();
            worker_locked.state = State::Idle;
    
            if last_instant.is_some() && 
                this_instant.duration_since(
                    (&last_instant.unwrap()).clone())
                    .unwrap().as_secs() <= config.update_interval {
                    drop(worker_locked);
                continue;
            }
            worker_locked.state = State::Running;
            drop(worker_locked);
    
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
            last_instant = Some(this_instant);
            if config.disable_cache {
                if let Some(last_path) = last_path {
                    if path != last_path {
                        match std::fs::remove_file(last_path) {
                            Ok(_) => {}
                            Err(e) => {
                                eprintln!("Could not removed cached image for some reason: {}. Skipping...", e);
                            }
                        }
                    }
                }
            }
            last_path = Some(path.clone());
            println!("New photo downloaded at: {}. Setting wallpaper...", path);
            
            match set_wallpaper(&path) {
                Ok(_) => {}
                Err(e) => {
                    eprintln!("Failed to set wallpaper: {}. Stopping...", e);
                    return;
                }
            }
    
            thread::sleep(Duration::from_secs(config.repeat_secs));
        }
    }

    pub fn send(&self, msg: Message) {
        self.sender.send(msg).unwrap();
    }

    pub fn meta_send(&self, msg: MetaMessage) {
        self.meta_sender.lock().unwrap().send(msg).unwrap();
    }
}

