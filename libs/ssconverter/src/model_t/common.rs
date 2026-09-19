pub fn convert_vec2t(point2: &sspj::sstypes::SsPoint2) -> ssab::ssab::ss::format::Vec2T {
    let mut vec = ssab::ssab::ss::format::Vec2T::default();
    vec.v1 = point2.x;
    vec.v2 = point2.y;
    vec
}
pub fn convert_flat_list_from_vec3t_list(point3_list: &[sspj::sstypes::SsPoint3]) -> Vec<u16> {
    point3_list.iter().flat_map(|point3| [point3.x as u16, point3.y as u16, point3.z as u16]).collect::<Vec<u16>>()
}

pub fn convert_min_max2t(x_min: f32, y_min: f32, x_max: f32, y_max: f32) -> ssab::ssab::ss::format::MinMax2T {
    let mut min_max2 = ssab::ssab::ss::format::MinMax2T::default();
    min_max2.x_min = x_min;
    min_max2.y_min = y_min;
    min_max2.x_max = x_max;
    min_max2.y_max = y_max;

    min_max2
}

pub fn convert_color(c: &sspj::sstypes::SsU8Color) -> u32 {
    ((c.a as u32) << 24) | ((c.r as u32) << 16) | ((c.g as u32) << 8) | (c.b as u32)
}
