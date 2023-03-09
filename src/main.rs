mod color;
mod dither;
mod utility;
mod worker;

use std::sync::Arc;
use std::thread::{self};
use std::time::Instant;

use color::Color;
use image::{self, ImageBuffer, Rgba};

use crate::dither::{Dither, DitherJob};
use crate::worker::WorkerCollection;

const THREAD_COUNT: u32 = 8;

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
    let now = Instant::now();
    // let (width, height) = (0x100, 0x100);
    let (width, height) = (512, 512);

    // // TODO: parse command line arguments instead of hard-linking a path
    let img = match image::open("images/selfie.jpg") {
        Ok(img) => {
            img.resize(width, height, image::imageops::FilterType::Nearest)
        }
        Err(error) => panic!("{}", error),
    };

    let reference_image = Arc::new(img);
    let mut output = ImageBuffer::<Rgba<u8>, Vec<u8>>::new(width, height);

    let mut manager = worker::Manager::<DitherJob>::new(THREAD_COUNT);
    let worker_count = manager.worker_count;

    let dither = Arc::new(Dither::new(
        worker_count,
        reference_image,
        &PALETTE,
        GAMMA,
        SPREAD,
    ));

    manager.set_workers(&|id| {
        let dither = Arc::clone(&dither);
        thread::spawn(move || dither.dither_section(id))
    });

    manager.collect(&mut output);

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
