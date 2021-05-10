use std::{thread, time::Duration};

use awc::Worker;

fn main() {
    let worker = Worker::new();
    loop {
        thread::sleep(Duration::from_secs(1));
    }
}
