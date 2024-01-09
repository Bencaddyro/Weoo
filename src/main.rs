// #![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use crate::{geolib::Container, iolib::load_database};
use arboard::Clipboard;
use chrono::prelude::*;
use eframe::egui;

use geolib::{get_current_container, ProcessedPosition, SpaceTimePosition, Vec3d};
use mainlib::{Path, Target};
use once_cell::sync::Lazy;
use regex::Regex;
use std::{
    collections::{BTreeMap, HashMap},
    f64::{consts::PI, NAN},
};
use uuid::Uuid;

mod geolib;
mod guilib;
mod iolib;
mod mainlib;

// Somewhere on Daymar
// Coordinates: x:-18930379393.98 y:-2610297380.75 z:210614.307494
// Coordinates: x:-18930479393.98 y:-2610297380.75 z:210614.307494
// Coordinates: x:-18930579393.98 y:-2610297380.75 z:210614.307494
// Coordinates: x:-18930679393.98 y:-2610297380.75 z:210614.307494
// Coordinates: x:-18930779393.98 y:-2610297380.75 z:210614.307494

static REFERENCE_TIME: Lazy<DateTime<Utc>> =
    Lazy::new(|| Utc.with_ymd_and_hms(2020, 1, 1, 0, 0, 0).unwrap());

fn main() -> eframe::Result<()> {
    let native_options = eframe::NativeOptions::default();
    eframe::run_native(
        "Weoo Nav Tool",
        native_options,
        Box::new(|cc| Box::new(MyEguiApp::new(cc))),
    )
}

type Paths = HashMap<String, Path>;
type Targets = Vec<Target>;
type Database = BTreeMap<String, Container>;

// #[derive(Default)]
struct MyEguiApp {
    // IO
    clipboard: Clipboard,
    space_time_position: SpaceTimePosition,
    path_name_io: String,

    // Data
    database: Database,

    // App State
    current_heading: f64,

    // Point history, Store all IO point from clipboard of map Input
    global_history_index: usize,
    global_history: Vec<ProcessedPosition>,

    // Paths
    global_paths: Paths,
    path_add_point: bool,
    path_selector: String,

    // Targets
    global_targets: Targets,
    target_selector_poi: String,
    target_selector_container: String,
}

impl MyEguiApp {
    fn new(_cc: &eframe::CreationContext<'_>) -> Self {
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
            global_paths: HashMap::from([("Self".to_string(), Path::new("Self".to_string()))]),
            global_targets: targets,
            path_selector: "Self".to_string(),
            path_add_point: true,
            target_selector_poi: String::new(),
            target_selector_container: String::new(),
            current_heading: NAN,
        }
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
        self.global_history.push(new_position.clone());
        self.global_history_index += 1;

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
        self.global_history.push(new_position.clone());
        self.global_history_index += 1;

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
            caps["x"].parse::<f64>().unwrap() / 1000.0,
            caps["y"].parse::<f64>().unwrap() / 1000.0,
            caps["z"].parse::<f64>().unwrap() / 1000.0,
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
    }
}
