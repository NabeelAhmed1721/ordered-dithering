use std::sync::Arc;

use crate::color::Color;
use crate::utility;

use image::{DynamicImage, GenericImageView, ImageBuffer, Rgba};

pub struct DitherJob {
    pub buffer: ImageBuffer<Rgba<u8>, Vec<u8>>,
    pub from: u32,
    pub to: u32,
}

const MATRIX_WIDTH: u32 = 8;
const MATRIX: [u16; 64] = [
    0, 32, 8, 40, 2, 34, 10, 42, 48, 16, 56, 24, 50, 18, 58, 26, 12, 44, 4, 36,
    14, 46, 6, 38, 60, 28, 52, 20, 62, 30, 54, 22, 3, 35, 11, 43, 1, 33, 9, 41,
    51, 19, 59, 27, 49, 17, 57, 25, 15, 47, 7, 39, 13, 45, 5, 37, 63, 31, 55,
    23, 61, 29, 53, 21,
];

// needs to return image buffer with start, end.
pub fn dither_image(
    thread_count: u32,
    thread_id: u32,
    image: Arc<DynamicImage>,
    gamma: f32,
    spread: f32,
    palette: &[Color],
) -> DitherJob {
    let width: u32 = image.width();
    let height: u32 = image.height();
    let area = width * height;

    // TODO: dynamic buffer size per thread to optimize memory use
    let mut thread_buffer =
        ImageBuffer::<Rgba<u8>, Vec<u8>>::new(width, height);
    let thread_location_start = (area / thread_count) * thread_id;
    let mut thread_location_end = (area / thread_count) * (thread_id + 1) - 1;

    // check if last thread && if any jobs will go missing
    if thread_id == thread_count - 1 && area % thread_count != 0 {
        // make sure last thread completes remaining jobs
        thread_location_end = area - 1;
    }

    for i in thread_location_start..thread_location_end {
        let (x, y) = (i % width, i / width);
        let [r, g, b, _] = image.get_pixel(x, y).0;

        let bay = utility::map_range_value(bayer(x, y), (0, 64), (0, 255));

        let quantize_value = |c: u8| -> u8 {
            f32::min(
                255.0 * f32::powf(f32::from(c) / 255.0, gamma)
                    + spread * f32::from(bay),
                255.0,
            ) as u8
        };

        let query_color =
            Color(quantize_value(r), quantize_value(g), quantize_value(b));

        let closest_color = match query_color.find_closest_color(palette) {
            Some(color) => color,
            None => Color(0, 0, 0),
        };

        thread_buffer.put_pixel(
            x,
            y,
            Rgba([closest_color.0, closest_color.1, closest_color.2, 255]),
        );
    }

    return DitherJob {
        buffer: thread_buffer,
        from: thread_location_start,
        to: thread_location_end,
    };
}

// TODO: organize different pre-computed bayer matrix sizes
fn bayer(x: u32, y: u32) -> u16 {
    // bayer matrix 8x8
    let pos =
        x % MATRIX_WIDTH + ((y * MATRIX_WIDTH) % (MATRIX_WIDTH * MATRIX_WIDTH));
    MATRIX[pos as usize]
}
