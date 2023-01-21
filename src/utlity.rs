use image::Primitive;

pub fn map_range_value<T: Primitive>(value: T, range_in: (T, T), range_out: (T, T)) -> T {
    return (value - range_in.0) * (range_out.1 - range_out.0) / (range_in.1 - range_in.0)
        + range_out.0;
}
