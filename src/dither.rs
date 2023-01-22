use crate::color::Color;
use crate::utlity;

use image::{DynamicImage, GenericImageView, ImageBuffer, Rgba};

const MATRIX_WIDTH: u32 = 8;
const MATRIX: [u16; 64] = [
    0, 32, 8, 40, 2, 34, 10, 42, 48, 16, 56, 24, 50, 18, 58, 26, 12, 44, 4, 36, 14, 46, 6, 38, 60,
    28, 52, 20, 62, 30, 54, 22, 3, 35, 11, 43, 1, 33, 9, 41, 51, 19, 59, 27, 49, 17, 57, 25, 15,
    47, 7, 39, 13, 45, 5, 37, 63, 31, 55, 23, 61, 29, 53, 21,
];

pub fn dither_image(
    image: &DynamicImage,
    gamma: f32,
    spread: f32,
    // TODO: add generic support
    output: &mut ImageBuffer<Rgba<u8>, Vec<u8>>,
    palette: &[Color],
) {
    // let width: u32 = output.width();
    // let height: u32 = output.height();

    for (x, y, color) in image.pixels() {
        let [r, g, b, _] = color.0;

        let bay = utlity::map_range_value(bayer(x, y), (0, 64), (0, 255));

        let quantize_value = |c: u8| -> u8 {
            f32::min(
                255.0 * f32::powf(f32::from(c) / 255.0, gamma) + spread * f32::from(bay),
                255.0,
            ) as u8
        };

        let query_color = Color(quantize_value(r), quantize_value(g), quantize_value(b));

        let closest_color = match query_color.find_closest_color(palette) {
            Some(color) => color,
            None => Color(0, 0, 0),
        };

        output.put_pixel(
            x,
            y,
            Rgba([closest_color.0, closest_color.1, closest_color.2, 255]),
        );
    }
}

// TODO: organize different pre-computed bayer matrix sizes
fn bayer(x: u32, y: u32) -> u16 {
    // bayer matrix 8x8
    let pos = x % MATRIX_WIDTH + ((y * MATRIX_WIDTH) % (MATRIX_WIDTH * MATRIX_WIDTH));
    MATRIX[pos as usize]
}
