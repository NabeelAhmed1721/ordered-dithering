#[derive(Clone, Copy)]
pub struct Color(pub u8, pub u8, pub u8);

impl Color {
    /// Calculates the squared distance between two colors.
    ///
    /// Square root is *not* applied.
    pub fn get_distance(&self, delta: Color) -> u32 {
        let Color(r1, g1, b1) = self;
        let Color(r2, g2, b2) = delta;

        let dist_square = |x1: u8, x2: u8, weight: f32| -> u32 {
            let difference = i32::from(x1) - i32::from(x2);
            let weight: u32 = (difference as f32 * weight).abs() as u32;
            return weight.pow(2);
        };

        // Added color weights to adjust for luminance
        let r = dist_square(*r1, r2, 0.299);
        let g = dist_square(*g1, g2, 0.587);
        let b = dist_square(*b1, b2, 0.114);

        return r + g + b;
    }

    pub fn find_closest_color(&self, palette: &[Color]) -> Option<Color> {
        let mut result = None;
        let mut least_distance = u32::MIN;

        for delta in palette {
            let distance = self.get_distance(*delta);

            if result.is_none() || least_distance > distance {
                least_distance = distance;
                result = Some(*delta);
            }
        }

        return result;
    }
}
