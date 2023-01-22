mod color;
mod dither;
mod utlity;

use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};

use color::Color;
use image::{self, ImageBuffer, Rgba};

struct ThreadWorker {
    thread_handler: JoinHandle<()>,
    thread_id: u32,
}

const THREAD_COUNT: u32 = 2;

// TODO: find a way to send palette arguments
// pre-generated 8 bit color palette
const PALETTE: [Color; 8] = [
    Color(0, 0, 0),
    Color(255, 0, 0),
    Color(0, 255, 0),
    Color(0, 0, 255),
    Color(255, 255, 0),
    Color(255, 0, 255),
    Color(0, 255, 255),
    Color(255, 255, 255),
];

const GAMMA: f32 = 1.8;
const SPREAD: f32 = 0.5;

fn main() {
    use std::time::Instant;
    let now = Instant::now();
    let (width, height) = (512, 512);

    // TODO: parse command line arguments instead of hard-linking a path
    let img = image::open("images/selfie.jpg");

    let img = match img {
        Ok(img) => img,
        Err(error) => panic!("{}", error),
    }
    .resize(width, height, image::imageops::FilterType::Nearest);

    let img = Arc::new(Mutex::new(img));

    let output = Arc::new(Mutex::new(ImageBuffer::<Rgba<u8>, Vec<u8>>::new(
        width, height,
    )));

    let mut thread_workers: Vec<ThreadWorker> = vec![];

    for thread_id in 0..THREAD_COUNT {
        let thread_img = Arc::clone(&img);
        let mut thread_output = Arc::clone(&output);

        let thread_handler = thread::spawn(move || {
            dither::dither_image(
                THREAD_COUNT,
                thread_id,
                &thread_img.lock().unwrap(),
                GAMMA,
                SPREAD,
                &mut thread_output,
                &PALETTE,
            );
        });

        thread_workers.push(ThreadWorker {
            thread_handler,
            thread_id,
        });
    }

    for thread_worker in thread_workers {
        let ThreadWorker {
            thread_handler,
            thread_id,
        } = thread_worker;

        match thread_handler.join() {
            Ok(_) => println!("Thread #{} complete.", thread_id + 1),
            Err(error) => panic!("{:?}", error),
        }
    }

    println!("All threads complete.");

    let output = output.lock().unwrap();

    let save = image::save_buffer(
        "images/out.png",
        &output,
        output.width(),
        output.height(),
        image::ColorType::Rgba8,
    );

    match save {
        Ok(_) => println!("Image successfully saved."),
        Err(error) => panic!("{}", error),
    }

    let elapsed = now.elapsed();
    println!("Elapsed: {:.2?}", elapsed);
}

// TODO: add tests
