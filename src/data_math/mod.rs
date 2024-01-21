// Weoo, a StarCitizen navigation tool
// Copyright (C) 2024 Benoît Fournier benoit.fournier@clever-cloud.com
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.
//

/// Boring traits implementation
mod traits;
pub use traits::*;

use crate::prelude::*;

use chrono::{DateTime, Utc};
use egui::Color32;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Default, Copy, Clone, PartialEq, Deserialize, Serialize)]
pub struct SpaceTimePosition {
    pub coordinates: Vec3d,
    pub timestamp: DateTime<Utc>,
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

#[derive(Debug, Deserialize, Serialize, Copy, Clone, PartialEq, Default)]
pub struct Vec3d {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

#[derive(Debug, Deserialize, Serialize, Copy, Clone, PartialEq, Default)]
pub struct Vec4d {
    pub w: f64,
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl Vec4d {
    pub fn new(w: f64, x: f64, y: f64, z: f64) -> Vec4d {
        Vec4d { w, x, y, z }
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
    // angle is radian !
    pub fn rotate(&self, angle: f64) -> Vec3d {
        let angle = if angle.is_nan() { 0.0 } else { angle };
        Vec3d {
            x: angle.cos() * self.x - angle.sin() * self.y,
            y: angle.sin() * self.x + angle.cos() * self.y,
            z: self.z,
        }
    }
    pub fn latitude(&self) -> f64 {
        (self.z / self.norm()).asin()
    }
    pub fn longitude(&self) -> f64 {
        self.x.atan2(self.y) * -1.0
    }
    pub fn altitude(&self, sea_level: f64) -> f64 {
        self.norm() - sea_level
    }

    pub fn transform_to_local(&self, time_elapsed: f64, container: &OldContainer) -> Vec3d {
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

#[derive(Debug, Deserialize, Serialize, Default)]
pub struct NewDatabase {
    pub containers: BTreeMap<String, Container>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(untagged)]
pub enum Container {
    Node(Node),
    System(System),
}

#[derive(Debug, Deserialize, Serialize, Default)]
pub struct System {
    /// System name
    pub name: String,
    /// List of all POIs inside this container
    pub pois: Vec<PointOfInterest>,
    /// List of all containers inside this container
    pub containers: Vec<String>,
}

#[derive(Debug, Deserialize, Serialize, Default)]
pub struct Node {
    /// Container name
    pub name: String,
    /// Container's parent name
    pub parent: String,
    /// Vector to describe translation from parent container
    pub from_parent_coordinates: Vec3d,
    /// Quaternion to describe rotation from parent container (rad/sec)
    pub from_parent_rotation: Vec4d,
    /// Quaternion to define self rotation (rad/sec)
    pub self_rotation: Vec4d,
    /// Rotation offset in radians
    pub rotation_offset: f64,
    /// List of all POIs inside this container
    pub pois: BTreeMap<String, PointOfInterest>,
    /// List of all containers inside this container
    pub containers: Vec<String>,
}

#[derive(Debug, Deserialize, Serialize, Default)]
pub struct PointOfInterest {
    /// Point of Interest name
    pub name: String,
    /// Container name
    pub parent: String,
    /// Vector to describe translation from parent container
    pub coordinates: Vec3d,
    /// Is there a quantum marker for this point
    #[serde(default)]
    pub marker: bool,
}

#[derive(Debug, Deserialize, Serialize, Clone, Default, PartialEq)]
pub struct OldContainer {
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
    pub poi: BTreeMap<String, OldPoi>,
}
#[derive(Debug, Deserialize, Serialize, Clone, Default, PartialEq)]
pub struct OldPoi {
    pub name: String,
    pub container: String,
    pub coordinates: Vec3d,
    pub quaternions: Option<Vec4d>,
    pub marker: Option<bool>,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
    pub altitude: Option<f64>,
}

pub fn poi_to_processed_point(p: &OldPoi, database: &Database) -> ProcessedPosition {
    let sea_level = database.get(&p.container).unwrap().radius_body;
    ProcessedPosition {
        space_time_position: SpaceTimePosition::default(),
        local_coordinates: p.coordinates,
        time_elapsed: 0.0,
        container_name: p.container.to_string(),
        name: p.name.to_string(),
        latitude: p.coordinates.latitude(),
        longitude: p.coordinates.longitude(),
        altitude: p.coordinates.altitude(sea_level),
        color: None,
    }
}

pub fn get_current_container(
    pos: &Vec3d,
    database: &BTreeMap<String, OldContainer>,
) -> OldContainer {
    let mut current_container = OldContainer {
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
