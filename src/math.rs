#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Vec3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Vec3 {
    pub const fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }

    pub fn length_squared(self) -> f32 {
        self.x * self.x + self.y * self.y + self.z * self.z
    }

    pub fn length(self) -> f32 {
        self.length_squared().sqrt()
    }

    pub fn normalized(self) -> Self {
        let length = self.length();
        if length <= f32::EPSILON {
            Self::new(0.0, 0.0, 0.0)
        } else {
            self * (1.0 / length)
        }
    }
}

impl std::ops::Add for Vec3 {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self::new(self.x + rhs.x, self.y + rhs.y, self.z + rhs.z)
    }
}

impl std::ops::AddAssign for Vec3 {
    fn add_assign(&mut self, rhs: Self) {
        self.x += rhs.x;
        self.y += rhs.y;
        self.z += rhs.z;
    }
}

impl std::ops::Sub for Vec3 {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self::new(self.x - rhs.x, self.y - rhs.y, self.z - rhs.z)
    }
}

impl std::ops::Mul<f32> for Vec3 {
    type Output = Self;

    fn mul(self, rhs: f32) -> Self::Output {
        Self::new(self.x * rhs, self.y * rhs, self.z * rhs)
    }
}

pub fn rotate_y(point: Vec3, angle: f32) -> Vec3 {
    let sin = angle.sin();
    let cos = angle.cos();
    Vec3::new(
        point.x * cos - point.z * sin,
        point.y,
        point.x * sin + point.z * cos,
    )
}

pub fn forward(heading: f32) -> Vec3 {
    Vec3::new(heading.sin(), 0.0, heading.cos())
}

#[cfg(test)]
mod tests {
    use super::{Vec3, forward, rotate_y};

    #[test]
    fn rotate_y_turns_forward_to_right() {
        let rotated = rotate_y(Vec3::new(0.0, 0.0, 1.0), std::f32::consts::FRAC_PI_2);
        assert!((rotated.x + 1.0).abs() < 0.001);
        assert!(rotated.z.abs() < 0.001);
    }

    #[test]
    fn normalized_zero_vector_is_stable() {
        assert_eq!(
            Vec3::new(0.0, 0.0, 0.0).normalized(),
            Vec3::new(0.0, 0.0, 0.0)
        );
    }

    #[test]
    fn forward_points_down_positive_z_at_zero_heading() {
        let value = forward(0.0);
        assert!((value.x - 0.0).abs() < 0.001);
        assert!((value.z - 1.0).abs() < 0.001);
    }
}
