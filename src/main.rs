use std::process;

use awc::{Config, DEFAULT_CONFIG_PATH, Message, MetaMessage, Worker};
use fltk::{app::App, button::Button, prelude::*, window::Window};

fn main() {
    let mut config = match Config::from_path(DEFAULT_CONFIG_PATH) {
        Ok(config) => config,
        Err(e) => {
            eprintln!("Failed to create (or load) config: {}", e);
            process::exit(1);
        }
    };
    let worker = Worker::new();
    let app = App::default();
    let mut window = Window::new(100, 100, 400, 300, "Automagic Wallpaper Changer");
    let mut start_worker = Button::new(10, 10, 100, 30, "Start");
    {
        let worker = worker.clone();
        start_worker.set_callback(move |_| {
            worker.lock().unwrap().meta_send(MetaMessage::Start);
        });
    }
    let mut redownload = Button::new(120, 10, 200, 30, "Refresh wallpaper");
    {
        let worker = worker.clone();
        redownload.set_callback(move |_| {
            println!("Sending redownload...");
            worker.lock().unwrap().send(Message::Redownload);
        });
    }
    
    window.end();
    window.show();
    app.run().unwrap();
}
