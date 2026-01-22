pub fn convert_vec2t(point2: &sspj::sstypes::SsPoint2) -> ssab::ssab::Vec2T {
    let mut vec = ssab::ssab::Vec2T::default();
    vec.v1 = point2.x;
    vec.v2 = point2.y;
    vec
}
pub fn convert_vec2t_list(point2_list: &[sspj::sstypes::SsPoint2]) -> Vec<ssab::ssab::Vec2T> {
    point2_list.iter().map(|point2| convert_vec2t(point2)).collect::<Vec<ssab::ssab::Vec2T>>()
}

pub fn convert_flat_list_from_vec3t_list(point3_list: &[sspj::sstypes::SsPoint3]) -> Vec<u16> {
    point3_list.iter().flat_map(|point3| [point3.x as u16, point3.y as u16, point3.z as u16]).collect::<Vec<u16>>()
}

pub fn convert_rectt(x1: f32, y1: f32, x2: f32, y2: f32) -> ssab::ssab::RectT {
    let mut rect = ssab::ssab::RectT::default();
    rect.x1 = x1;
    rect.y1 = y1;
    rect.x2 = x2;
    rect.y2 = y2;

    rect
}

pub fn convert_color(c: &sspj::sstypes::SsU8Color) -> u32 {
    ((c.a as u32) << 24) | ((c.r as u32) << 16) | ((c.g as u32)<< 8) | (c.b as u32)
}
