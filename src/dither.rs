use std::sync::Arc;

use crate::color::Color;
use crate::utility;

use image::{DynamicImage, GenericImageView, ImageBuffer, Rgba};

const MATRIX_WIDTH: u32 = 8;
const MATRIX: [u16; 64] = [
    0, 32, 8, 40, 2, 34, 10, 42, 48, 16, 56, 24, 50, 18, 58, 26, 12, 44, 4, 36,
    14, 46, 6, 38, 60, 28, 52, 20, 62, 30, 54, 22, 3, 35, 11, 43, 1, 33, 9, 41,
    51, 19, 59, 27, 49, 17, 57, 25, 15, 47, 7, 39, 13, 45, 5, 37, 63, 31, 55,
    23, 61, 29, 53, 21,
];

#[derive(Debug)]
pub struct DitherJob {
    pub buffer: ImageBuffer<Rgba<u8>, Vec<u8>>,
    pub from: u32,
    pub to: u32,
}

pub struct Dither<'life> {
    worker_count: u32,
    reference_image: Arc<DynamicImage>,
    palette: &'life [Color],
    gamma: f32,
    spread: f32,
}

impl<'life> Dither<'life> {
    pub fn new(
        worker_count: u32,
        reference_image: Arc<DynamicImage>,
        palette: &'life [Color],
        gamma: f32,
        spread: f32,
    ) -> Self {
        Dither {
            worker_count,
            reference_image,
            palette,
            gamma,
            spread,
        }
    }

    pub fn dither_section(&self, worker_id: u32) -> DitherJob {
        let reference_image = Arc::clone(&self.reference_image);
        let (width, height) =
            (reference_image.width(), reference_image.height());
        let area = width * height;

        // TODO: dynamic buffer size per thread to optimize memory use
        let mut thread_buffer =
            ImageBuffer::<Rgba<u8>, Vec<u8>>::new(width, height);

        let thread_location_start = (area / self.worker_count) * worker_id;

        let mut thread_location_end =
            (area / self.worker_count) * (worker_id + 1) - 1;

        // check if last thread && if any jobs will go missing
        if worker_id == self.worker_count - 1 && area % self.worker_count != 0 {
            // make sure last thread completes remaining jobs
            thread_location_end = area - 1;
        }

        for i in thread_location_start..thread_location_end {
            let (x, y) = (i % width, i / width);
            let [r, g, b, _] = reference_image.get_pixel(x, y).0;

            let bay = utility::map_range_value(
                Dither::get_bayer(x, y),
                (0, 64),
                (0, 255),
            );

            let quantize_value = |c: u8| -> u8 {
                f32::min(
                    255.0 * f32::powf(f32::from(c) / 255.0, self.gamma)
                        + self.spread * f32::from(bay),
                    255.0,
                ) as u8
            };

            let query_color =
                Color(quantize_value(r), quantize_value(g), quantize_value(b));

            let closest_color =
                match query_color.find_closest_color(self.palette) {
                    Some(color) => color,
                    None => Color(0, 0, 0),
                };

            thread_buffer.put_pixel(
                x,
                y,
                Rgba([closest_color.0, closest_color.1, closest_color.2, 255]),
            );
        }

        DitherJob {
            buffer: thread_buffer,
            from: thread_location_start,
            to: thread_location_end,
        }
    }

    fn get_bayer(x: u32, y: u32) -> u16 {
        // TODO: organize different pre-computed bayer matrix sizes
        // 8x8 Bayer Matrix
        let pos = x % MATRIX_WIDTH
            + ((y * MATRIX_WIDTH) % (MATRIX_WIDTH * MATRIX_WIDTH));

        MATRIX[pos as usize]
    }
}
