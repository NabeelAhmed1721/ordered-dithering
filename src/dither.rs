use crate::color::Color;
use crate::utlity;

use image::{DynamicImage, GenericImageView, ImageBuffer, Rgba};

pub fn dither_image(
    image: &DynamicImage,
    // TODO: add generic support
    output: &mut ImageBuffer<Rgba<u8>, Vec<u8>>,
    palette: &[Color],
) {
    let width: u32 = output.width();
    let height: u32 = output.height();

    // let brightness = (r as u32 + g as u32 + b as u32) / 3;
    // let gamma = 2.4;
    let gamma = 2.3;
    let spread = 0.5;

    for (x, y, color) in image.pixels() {
        let [r, g, b, _] = color.0;

        let bay = utlity::map_range_value(bayer(x, y), (0, 64), (0, 255));

        bayer(0, 0);

        // TODO: improve code readablity
        let query_color = Color(
            f32::min(
                255.0 * f32::powf(r as f32 / 255.0, gamma) + spread * bay as f32,
                255.0,
            ) as u8,
            f32::min(
                255.0 * f32::powf(g as f32 / 255.0, gamma) + spread * bay as f32,
                255.0,
            ) as u8,
            f32::min(
                255.0 * f32::powf(b as f32 / 255.0, gamma) + spread * bay as f32,
                255.0,
            ) as u8,
        );

        let closest_color = match query_color.find_closest_color(palette) {
            Some(color) => color,
            None => Color(0, 0, 0),
        };

        output.put_pixel(
            x,
            y,
            Rgba([closest_color.0, closest_color.1, closest_color.2, 255]),
        );

        // TODO: move this to its own module
        print!("\x1B[2J\x1B[1;1H");
        let loc = x % width + ((y * width) % (width * width));
        let progress = ((loc as f32 / ((width as f32 * height as f32) - 1.0)) * 100.0) as u32;
        println!("progress {}%", progress);
    }
}

// TODO: organize different pre-computed bayer matrix sizes
fn bayer(x: u32, y: u32) -> u32 {
    // bayer matrix 8x8
    let matrix: [u32; 64] = [
        0, 32, 8, 40, 2, 34, 10, 42, 48, 16, 56, 24, 50, 18, 58, 26, 12, 44, 4, 36, 14, 46, 6, 38,
        60, 28, 52, 20, 62, 30, 54, 22, 3, 35, 11, 43, 1, 33, 9, 41, 51, 19, 59, 27, 49, 17, 57,
        25, 15, 47, 7, 39, 13, 45, 5, 37, 63, 31, 55, 23, 61, 29, 53, 21,
    ];
    let matrix_width: u32 = 8;
    let pos = x % matrix_width + ((y * matrix_width) % (matrix_width * matrix_width));

    matrix[pos as usize]
}
