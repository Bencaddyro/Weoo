use chrono::Utc;
use egui::Color32;
use egui_plot::MarkerShape;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::ops::{Add, Sub};

#[derive(Debug, Default, Copy, Clone, PartialEq, Deserialize, Serialize)]
pub struct SpaceTimePosition {
    pub coordinates: Vec3d,
    pub timestamp: chrono::DateTime<Utc>,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct ProcessedPosition {
    pub space_time_position: SpaceTimePosition,
    pub local_coordinates: Vec3d,
    pub time_elapsed: f64,
    pub container_name: String,
    pub name: String,
    pub latitude: f64,
    pub longitude: f64,
    pub altitude: f64,
    pub color: Option<Color32>,
}

#[derive(Debug, Clone)]
pub struct Path {
    pub name: String,
    pub history: Vec<ProcessedPosition>,
    pub color: Color32,
    pub shape: MarkerShape,
    pub radius: f32,
}

#[derive(Debug, Deserialize, Serialize, Copy, Clone, PartialEq, Default)]
pub struct Vec3d {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl Path {
    pub fn new() -> Path {
        Path {
            name: "Self".to_string(),
            history: Vec::new(),
            color: Color32::DARK_GRAY,
            shape: MarkerShape::Circle,
            radius: 3.0,
        }
    }
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
        (self.z / self.norm()).asin()
    }
    pub fn longitude(&self) -> f64 {
        self.x.atan2(self.y) * -1.0
    }
    pub fn altitude(&self, container: &Container) -> f64 {
        self.norm() - container.radius_body
    }

    pub fn transform_to_local(&self, time_elapsed: f64, container: &Container) -> Vec3d {
        let rotation_speed_in_degrees_per_second = 0.1 * (1.0 / container.rotation_speed);
        let rotation_state_in_degrees = (rotation_speed_in_degrees_per_second * time_elapsed
            + container.rotation_adjust)
            % 360.0;
        (*self - container.coordinates).rotate((-rotation_state_in_degrees).to_radians())
    }

    pub fn loxodromie_to(&self, target: Vec3d) -> f64 {
        // let a = ((target.longitude().to_degrees() - self.longitude().to_degrees())
        // / ((PI / 4.0 + self.latitude() / 2.0).tan().ln()
        // - (PI / 4.0 + target.latitude() / 2.0).tan().ln()))
        // .atan();
        let x = target.latitude().cos() * (target.longitude() - self.longitude()).sin();
        let y = self.latitude().cos() * target.latitude().sin()
            - self.latitude().sin()
                * target.latitude().cos()
                * (target.longitude() - self.longitude()).cos();
        x.atan2(y)
        // let c = y.atan2(x);
        // println!("{a} {b} {c}");
    }
}

#[derive(Debug, Deserialize, Serialize, Copy, Clone, PartialEq, Default)]
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
#[derive(Debug, Deserialize, Serialize, Clone, Default, PartialEq)]
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
    pub poi: BTreeMap<String, Poi>,
}
#[derive(Debug, Deserialize, Serialize, Clone, Default, PartialEq)]
pub struct Poi {
    pub name: String,
    pub container: String,
    pub coordinates: Vec3d,
    pub quaternions: Option<Vec4d>,
    pub marker: Option<bool>,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
    pub altitude: Option<f64>,
}

pub fn get_current_container(pos: &Vec3d, database: &BTreeMap<String, Container>) -> Container {
    let mut current_container = Container {
        name: "Space".to_string(),
        coordinates: Vec3d::new(0.0, 0.0, 0.0),
        quaternions: Vec4d::new(0.0, 0.0, 0.0, 0.0),
        marker: false,
        radius_om: 0.0,
        radius_body: 0.0,
        radius_arrival: 0.0,
        time_lines: 0.0,
        rotation_speed: 0.0,
        rotation_adjust: 0.0,
        orbital_radius: 0.0,
        orbital_speed: 0.0,
        orbital_angle: 0.0,
        grid_radius: 0.0,
        poi: BTreeMap::new(),
    };

    for c in database.values() {
        if (c.coordinates - *pos).norm() <= 3.0 * c.radius_om {
            current_container = c.clone();
        }
    }
    current_container
}
