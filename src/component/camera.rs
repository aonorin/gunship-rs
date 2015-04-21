use polygon::camera::Camera;
use math::point::Point;
use math::matrix::Matrix4;

use entity::Entity;

pub struct CameraManager {
    cameras: Vec<Camera>
}

impl CameraManager {
    pub fn new() -> CameraManager {
        CameraManager {
            cameras: Vec::new()
        }
    }

    pub fn create(&mut self, entity: Entity, fov: f32, aspect: f32, near: f32, far: f32,) -> &mut Camera {
        self.cameras.push(Camera {
            fov: fov,
            aspect: aspect,
            near: near,
            far: far,

            position: Point::origin(),
            rotation: Matrix4::identity()
        });

        let index = self.cameras.len() - 1;
        &mut self.cameras[index]
    }

    pub fn cameras(&self) -> &Vec<Camera> {
        &self.cameras
    }

    pub fn cameras_mut(&mut self) -> &mut Vec<Camera> {
        &mut self.cameras
    }
}