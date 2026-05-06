pub use glam::{Vec3, Vec4, Mat4, Quat};

pub type Vector3 = Vec3;
pub type Matrix4 = Mat4;
pub type Quaternion = Quat;

pub fn look_at(eye: Vector3, center: Vector3, up: Vector3) -> Matrix4 {
    Mat4::look_at_rh(eye, center, up)
}

pub fn perspective(
    fovy: f32,
    aspect: f32,
    near: f32,
    far: f32,
) -> Matrix4 {
    Mat4::perspective_rh(fovy, aspect, near, far)
}

pub fn orthographic(
    left: f32,
    right: f32,
    bottom: f32,
    top: f32,
    near: f32,
    far: f32,
) -> Matrix4 {
    Mat4::orthographic_rh(left, right, bottom, top, near, far)
}

pub fn translate(translation: Vector3) -> Matrix4 {
    Mat4::from_translation(translation)
}

pub fn scale(scale: Vector3) -> Matrix4 {
    Mat4::from_scale(scale)
}

pub fn rotate(quat: Quaternion) -> Matrix4 {
    Mat4::from_quat(quat)
}
