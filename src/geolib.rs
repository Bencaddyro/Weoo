use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::ops::{Add, Sub};

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Default)]
pub struct Vec3d {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl Add for Vec3d {
    type Output = Self;
    fn add(self, other: Self) -> Self {
        Self {
            x: self.x + other.x,
            y: self.y + other.y,
            z: self.z + other.z,
        }
    }
}

impl Sub for Vec3d {
    type Output = Self;
    fn sub(self, other: Self) -> Self {
        Self {
            x: self.x - other.x,
            y: self.y - other.y,
            z: self.z - other.z,
        }
    }
}

impl Vec3d {
    pub fn new(x: f64, y: f64, z: f64) -> Vec3d {
        Vec3d { x, y, z }
    }
    pub fn norm(&self) -> f64 {
        (self.x * self.x + self.y * self.y + self.z * self.z).sqrt()
    }
    fn dot_product(&self, v: &Vec3d) -> f64 {
        self.x * v.x + self.y * v.y + self.z * v.z
    }
    fn angle_with(&self, v: &Vec3d) -> f64 {
        (self.dot_product(v) / (self.norm() + v.norm()))
            .acos()
            .to_degrees()
    }
    pub fn rotate(&self, angle: f64) -> Vec3d {
        let angle = if angle.is_nan() { 0.0 } else { angle };
        Vec3d::new(
            angle.cos() * self.x - angle.sin() * self.y,
            angle.sin() * self.x + angle.cos() * self.y,
            self.z,
        )
    }
    pub fn latitude(&self) -> f64 {
        (self.z / self.norm()).asin().to_degrees()
    }
    pub fn longitude(&self) -> f64 {
        self.x.atan2(self.y).to_degrees() * -1.0
    }
    pub fn height(&self, container: &Container) -> f64 {
        self.norm() - container.radius_body
    }

    pub fn transform_to_local(self, time_elapsed: f64, container: &Container) -> Vec3d {
        let rotation_speed_in_degrees_per_second = 0.1 * (1.0 / container.rotation_speed);
        let rotation_state_in_degrees = (rotation_speed_in_degrees_per_second * time_elapsed
            + container.rotation_adjust)
            % 360.0;
        (self - container.coordinates.clone()).rotate((-rotation_state_in_degrees).to_radians())
    }
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Default)]
pub struct Vec4d {
    pub qw: f64,
    pub qx: f64,
    pub qy: f64,
    pub qz: f64,
}

impl Vec4d {
    pub fn new(qw: f64, qx: f64, qy: f64, qz: f64) -> Vec4d {
        Vec4d { qw, qx, qy, qz }
    }
}
#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct Container {
    pub name: String,
    pub coordinates: Vec3d,
    pub quaternions: Vec4d,
    pub marker: bool,
    pub radius_om: f64,
    pub radius_body: f64,
    pub radius_arrival: f64,
    pub time_lines: f64,
    pub rotation_speed: f64,
    pub rotation_adjust: f64,
    pub orbital_radius: f64,
    pub orbital_speed: f64,
    pub orbital_angle: f64,
    pub grid_radius: f64,
    pub poi: HashMap<String, Poi>,
}

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct Poi {
    pub name: String,
    pub coordinates: Vec3d,
    pub quaternions: Vec4d,
    pub marker: bool,
    pub container: String,
}
