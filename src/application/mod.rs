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

mod mainlib;
pub use mainlib::*;

use crate::{prelude::*, REFERENCE_TIME};

use arboard::Clipboard;
use chrono::Utc;
use regex::Regex;
use std::{
    collections::HashMap,
    f64::{consts::PI, NAN},
    time::Duration,
};
use uuid::Uuid;

pub struct MyEguiApp {
    // IO
    pub clipboard: Clipboard,
    pub space_time_position: SpaceTimePosition,
    pub path_name_io: String,

    // Data
    pub database: Database,

    // App State
    pub current_heading: f64,

    // Point history, Store all IO point from clipboard of map Input
    pub global_history_index: usize,
    pub global_history: Vec<ProcessedPosition>,
    pub global_history_widget: bool,

    // Paths
    pub global_paths: Paths,
    pub path_add_point: bool,
    pub path_selector: String,

    // Targets
    pub global_targets: Targets,
    pub target_selector_poi: String,
    pub target_selector_container: String,
}

impl MyEguiApp {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        // Customize egui here with cc.egui_ctx.set_fonts and cc.egui_ctx.set_visuals.
        // Restore app state using cc.storage (requires the "persistence" feature).
        // Use the cc.gl (a glow::Context) to create graphics shaders and buffers that you can use
        // for e.g. egui::PaintCallback.

        let database = load_database();
        // Hardcode targets example for Daymar Rally
        // let target1 = database
        //     .get("Yela")
        //     .unwrap()
        //     .poi
        //     .get("BennyHenge")
        //     .unwrap()
        //     .to_owned();
        let target1 = database
            .get("Daymar")
            .unwrap()
            .poi
            .get("Better Shubin")
            .unwrap()
            .to_owned();
        let target2 = database
            .get("Daymar")
            .unwrap()
            .poi
            .get("Better Eager")
            .unwrap()
            .to_owned();
        let target3 = database
            .get("Daymar")
            .unwrap()
            .poi
            .get("Better Kudre")
            .unwrap()
            .to_owned();

        let targets = vec![
            Target::new(&target1, &database),
            Target::new(&target2, &database),
            Target::new(&target3, &database),
        ];

        let clipboard = match Clipboard::new() {
            Ok(clipboard) => clipboard,
            Err(e) => panic!("Error fetching clipoard: {e}"),
        };

        MyEguiApp {
            database,
            clipboard,
            space_time_position: SpaceTimePosition::default(),
            path_name_io: String::new(),
            global_history_index: 0,
            global_history: Vec::new(),
            global_history_widget: false,
            global_paths: HashMap::from([("Self".to_string(), Path::new("Self".to_string()))]),
            global_targets: targets,
            path_selector: "Self".to_string(),
            path_add_point: true,
            target_selector_poi: String::new(),
            target_selector_container: String::new(),
            current_heading: NAN,
        }
    }

    pub fn add_to_global(&mut self, position: &ProcessedPosition) {
        self.global_history.push(position.clone());
        self.global_history_index = self.global_history.len() - 1;
    }

    pub fn new_coordinates_input(&mut self) {
        // create ProcessedPosition from input
        let container =
            get_current_container(&self.space_time_position.coordinates, &self.database);
        let time_elapsed = (self.space_time_position.timestamp - *REFERENCE_TIME)
            .num_nanoseconds()
            .unwrap() as f64
            / 1e9;
        let local_coordinates = self
            .space_time_position
            .coordinates
            .transform_to_local(time_elapsed, &container);
        let (latitude, longitude, altitude);

        if container.name != "Space" {
            latitude = local_coordinates.latitude();
            longitude = local_coordinates.longitude();
            altitude = local_coordinates.altitude(container.radius_body);
        } else {
            latitude = NAN;
            longitude = NAN;
            altitude = NAN;
        }

        let name = "# ".to_owned() + &Uuid::new_v4().to_string()[9..18].to_uppercase();

        let new_position = ProcessedPosition {
            space_time_position: self.space_time_position,
            local_coordinates,
            time_elapsed,
            container_name: container.name,
            name,
            latitude,
            longitude,
            altitude,
            color: None,
        };

        // Add it to history
        self.add_to_global(&new_position);

        if self.path_add_point {
            if let Some(path) = self.global_paths.get_mut(&self.path_selector) {
                if path.history.is_empty() {
                    path.history.push(new_position.clone());
                } else {
                    path.history
                        .insert(path.current_index, new_position.clone());
                }
                path.current_index += 1;
            }
        }
    }

    pub fn new_coordinates_from_map(&mut self, latitude: f64, longitude: f64) {
        let latitude = latitude.clamp(-PI, PI);

        let container_name = "Daymar".to_string(); //TODO HardCode Daymar
        let altitude = 295.0;

        let timestamp = Utc::now();
        let z = altitude * latitude.sin();
        let x = -altitude * latitude.cos() * longitude.sin();
        let y = altitude * latitude.cos() * longitude.cos();

        let local_coordinates = Vec3d { x, y, z };

        let space_time_position = SpaceTimePosition {
            coordinates: Vec3d::default(),
            timestamp,
        };
        let time_elapsed = (timestamp - *REFERENCE_TIME).num_nanoseconds().unwrap() as f64 / 1e9;

        let name = "# ".to_owned() + &Uuid::new_v4().to_string()[9..18].to_uppercase();

        let new_position = ProcessedPosition {
            space_time_position,
            local_coordinates,
            time_elapsed,
            container_name,
            name,
            latitude,
            longitude,
            altitude,
            color: None,
        };

        // Add it to history
        self.add_to_global(&new_position);

        if self.path_add_point {
            if let Some(path) = self.global_paths.get_mut(&self.path_selector) {
                if path.history.is_empty() {
                    path.history.push(new_position.clone());
                } else {
                    path.history
                        .insert(path.current_index, new_position.clone());
                }
                path.current_index += 1;
            }
        }
    }

    fn get_space_time_position(&mut self) -> Option<SpaceTimePosition> {
        let content = match self.clipboard.get_text() {
            Ok(content) => content,
            Err(e) => {
                println!("Error fetching clipoard: {e}");
                String::new()
            }
        };
        let timestamp = Utc::now();

        let re = Regex::new(
                r"Coordinates: x:(?<x>-?[0-9]+\.[0-9]+) y:(?<y>-?[0-9]+\.[0-9]+) z:(?<z>-?[0-9]+\.[0-9]+)",
                    )
                .unwrap();
        let caps = re.captures(&content)?;
        let coordinates = Vec3d::new(
            caps["x"].parse::<f64>().unwrap(),
            caps["y"].parse::<f64>().unwrap(),
            caps["z"].parse::<f64>().unwrap(),
        );
        Some(SpaceTimePosition {
            coordinates,
            timestamp,
        })
    }
}

impl eframe::App for MyEguiApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if let Some(space_time_position_input) = self.get_space_time_position() {
            if space_time_position_input.coordinates != self.space_time_position.coordinates {
                // New Input !
                self.space_time_position = space_time_position_input;
                self.new_coordinates_input();
            }
        }

        let live_point = if self.global_history_index != 0 {
            self.global_history.get(self.global_history_index - 1)
        } else {
            None
        };

        // Update current heading based on last 2 point from global history
        if self.global_history.len() > 1 {
            if let [a, b] =
                &self.global_history[self.global_history.len() - 2..self.global_history.len()]
            {
                self.current_heading = a.local_coordinates.loxodromie_to(b.local_coordinates);
            };
        }

        // Update all NEW path
        for (_, path) in self.global_paths.iter_mut() {
            path.update(&self.database, live_point);
        }

        // Update all NEW target
        for target in self.global_targets.iter_mut() {
            target.update(&self.database, live_point);
        }

        // Display NEW everything
        self.display(ctx);

        // Auto refresh ~30 FPS
        ctx.request_repaint_after(Duration::from_millis(33));
    }
}
