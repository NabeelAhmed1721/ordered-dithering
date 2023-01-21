mod color;
mod dither;
mod utlity;

use color::Color;
use image::{self, ImageBuffer, Rgba};

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

    let mut output = ImageBuffer::<Rgba<u8>, Vec<u8>>::new(width, height);

    dither::dither_image(&img, GAMMA, SPREAD, &mut output, &PALETTE);

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
