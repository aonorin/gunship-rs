use std::ops::{Mul, MulAssign, Div, DivAssign, Neg, Add, AddAssign, Sub, SubAssign, Index, IndexMut};

use super::{IsZero, Dot};

#[repr(C)] #[derive(Debug, Clone, Copy, PartialEq)]
pub struct Vector3 {
    pub x: f32,
    pub y: f32,
    pub z: f32
}

impl Vector3 {
    pub fn new(x: f32, y: f32, z: f32) -> Vector3 {
        Vector3 {
            x: x,
            y: y,
            z: z
        }
    }

    pub fn zero() -> Vector3 {
        Vector3::new(0.0, 0.0, 0.0)
    }

    pub fn one() -> Vector3 {
        Vector3::new(1.0, 1.0, 1.0)
    }

    pub fn up() -> Vector3 {
        Vector3::new(0.0, 1.0, 0.0)
    }

    pub fn down() -> Vector3 {
        Vector3::new(0.0, -1.0, 0.0)
    }

    pub fn forward() -> Vector3 {
        Vector3::new(0.0, 0.0, -1.0)
    }

    /// TODO: Implement the `From` trait rather than making a separate method.
    pub fn from_slice(data: &[f32]) -> Vector3 {
        assert!(data.len() == 3);

        Vector3 {
            x: data[0],
            y: data[1],
            z: data[2],
        }
    }

    pub fn cross(first: Vector3, second: Vector3) -> Vector3 {
        Vector3 {
            x: first.y * second.z - first.z * second.y,
            y: first.z * second.x - first.x * second.z,
            z: first.x * second.y - first.y * second.x,
        }
    }

    /// Normalizes the vector, returning the old length.
    ///
    /// If the vector is the zero vector it is not altered.
    pub fn normalize(&mut self) -> f32 {
        if self.is_zero() {
            0.0
        } else {
            let magnitude = self.magnitude();
            let one_over_magnitude = 1.0 / magnitude;
            self.x *= one_over_magnitude;
            self.y *= one_over_magnitude;
            self.z *= one_over_magnitude;

            magnitude
        }
    }

    /// Returns the normalized version of the vector.
    ///
    /// If the vector is the zero vector a copy is returned.
    pub fn normalized(&self) -> Vector3 {
        if self.is_zero() {
            *self
        } else {
            let mut copy = *self;
            copy.normalize();
            copy
        }
    }

    pub fn is_normalized(&self) -> bool {
        (self.dot(self) - 1.0).is_zero()
    }

    pub fn magnitude(&self) -> f32 {
        self.magnitude_squared().sqrt()
    }

    pub fn magnitude_squared(&self) -> f32 {
        self.x * self.x + self.y * self.y + self.z * self.z
    }

    // Safely reinterprets a slice of Vector3s to a slice of f32s. This is a cheap operation and
    // does not copy any data.
    pub fn as_ref(vectors: &[Vector3]) -> &[f32] {
        unsafe {
            ::std::slice::from_raw_parts(
                vectors.as_ptr() as *const f32,
                vectors.len() * 3)
        }
    }

    // pub fn cross(&self, rhs: Vector3) -> Vector3 {
    //     Vector3::new(
    //         self.y * rhs.z - self.z * rhs.y,
    //         self.z * rhs.x - self.x * rhs.z,
    //         self.x * rhs.y - self.y * rhs.x)
    // }
}

impl Dot for Vector3 {
    type Output = f32;

    fn dot(self, rhs: Vector3) -> f32 {
        self.x * rhs.x + self.y * rhs.y + self.z * rhs.z
    }
}

impl Dot<[f32; 3]> for Vector3 {
    type Output = f32;

    fn dot(self, rhs: [f32; 3]) -> f32 {
        self.x * rhs[0] + self.y * rhs[1] + self.z * rhs[2]
    }
}

impl Dot<Vector3> for [f32; 3] {
    type Output = f32;

    fn dot(self, rhs: Vector3) -> f32 {
        rhs.dot(self)
    }
}

// impl<'a> Dot<&'a [f32; 3]> for Vector3 {
//     type Output = f32;
//
//     fn dot(self, rhs: &[f32; 3]) -> f32 {
//         self.dot(*rhs)
//     }
// }

impl AddAssign for Vector3 {
    fn add_assign(&mut self, rhs: Vector3) {
        self.x += rhs.x;
        self.y += rhs.y;
        self.z += rhs.z;
    }
}

impl Add for Vector3 {
    type Output = Vector3;

    fn add(mut self, rhs: Vector3) -> Vector3 {
        self += rhs;
        self
    }
}

impl SubAssign for Vector3 {
    fn sub_assign(&mut self, rhs: Vector3) {
        self.x -= rhs.x;
        self.y -= rhs.y;
        self.z -= rhs.z;
    }
}

impl Sub for Vector3 {
    type Output = Vector3;

    fn sub(mut self, rhs: Vector3) -> Vector3 {
        self -= rhs;
        self
    }
}

impl MulAssign for Vector3 {
    fn mul_assign(&mut self, rhs: Vector3) {
        self.x *= rhs.x;
        self.y *= rhs.y;
        self.z *= rhs.z;
    }
}

impl Mul for Vector3 {
    type Output = Vector3;

    fn mul(mut self, rhs: Vector3) -> Vector3 {
        self *= rhs;
        self
    }
}

impl MulAssign<f32> for Vector3 {
    fn mul_assign(&mut self, rhs: f32) {
        self.x *= rhs;
        self.y *= rhs;
        self.z *= rhs;
    }
}

impl Mul<f32> for Vector3 {
    type Output = Vector3;

    fn mul(mut self, rhs: f32) -> Vector3 {
        self *= rhs;
        self
    }
}

impl Mul<Vector3> for f32 {
    type Output = Vector3;

    fn mul(self, rhs: Vector3) -> Vector3 {
        rhs * self
    }
}

impl Neg for Vector3 {
    type Output = Vector3;

    fn neg(self) -> Vector3 {
        Vector3::new(-self.x, -self.y, -self.z)
    }
}

impl DivAssign<f32> for Vector3 {
    fn div_assign(&mut self, rhs: f32) {
        self.x /= rhs;
        self.y /= rhs;
        self.z /= rhs;
    }
}

impl Div<f32> for Vector3 {
    type Output = Vector3;

    fn div(mut self, rhs: f32) -> Vector3 {
        self /= rhs;
        self
    }
}

impl Div<Vector3> for f32 {
    type Output = Vector3;

    fn div(self, rhs: Vector3) -> Vector3 {
        rhs / self
    }
}

impl IsZero for Vector3 {
    fn is_zero(self) -> bool {
        self.dot(self).is_zero()
    }
}

// TODO: Is `usize` an appropriate index? Especially considering the valid values are 0..3?
impl Index<usize> for Vector3 {
    type Output = f32;

    fn index(&self, index: usize) -> &f32 {
        match index {
            0 => &self.x,
            1 => &self.y,
            2 => &self.z,
            // TODO: Use `unreachable()` intrinsic in release mode.
            _ => panic!("Index {} is out of bounds for Vector3", index),
        }
    }
}

impl IndexMut<usize> for Vector3 {
    fn index_mut(&mut self, index: usize) -> &mut f32 {
        match index {
            0 => &mut self.x,
            1 => &mut self.y,
            2 => &mut self.z,
            // TODO: Use `unreachable()` intrinsic in release mode.
            _ => panic!("Index {} is out of bounds for Vector3", index),
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, PartialOrd)]
pub struct Vector2 {
    pub x: f32,
    pub y: f32,
}

impl Vector2 {
    pub fn new(x: f32, y: f32) -> Vector2 {
        Vector2 {
            x: x,
            y: y,
        }
    }

    pub fn as_ref(vectors: &[Vector2]) -> &[f32] {
        use std::slice;

        unsafe {
            slice::from_raw_parts(
                vectors.as_ptr() as *const f32,
                vectors.len() * 2)
        }
    }
}
