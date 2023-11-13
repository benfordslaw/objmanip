use glam::Vec3;

pub fn winding(triangle: &[Vec3]) -> f32 {
    (triangle[1] - triangle[0])
        .cross(triangle[2] - triangle[0])
        .z
}
