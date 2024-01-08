// #![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use crate::{geolib::Container, iolib::load_database};
use arboard::Clipboard;
use chrono::prelude::*;
use eframe::egui;

use geolib::{get_current_container, ProcessedPosition, SpaceTimePosition, Vec3d};
use mainlib::{
    Path, Target, WidgetMap, WidgetPath, WidgetTarget, WidgetTargets, WidgetTopPosition,
};
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
// Coordinates: x:-18930379393.98 y:-2610297380.75 z:210614.307494
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

// #[derive(Default)]
struct MyEguiApp {
    // IO
    clipboard: Clipboard,
    space_time_position: SpaceTimePosition,
    path_name_io: String,

    // Data
    database: BTreeMap<String, Container>,

    // App State

    // Point history, Store all IO point from clipboard of map Input
    global_history_index: usize,
    global_history: Vec<ProcessedPosition>,
    global_paths: Paths,
    global_targets: Targets,

    path_selector: String,

    target_selector_poi: String,
    target_selector_container: String,

    // ---------------------- OLD
    index: usize,                 // TODO impl index on Path
    paths: HashMap<String, Path>, //check
    displayed_path: String,       //check

    // Gui component
    position: WidgetTopPosition,               //check
    targets: WidgetTargets,                    //check
    map: WidgetMap,                            //nothing here //TODO still need to be draw !
    targets_path: HashMap<String, WidgetPath>, //TODO widgetpath  need to be draw from Path
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
            WidgetTarget::new(target1, &database),
            WidgetTarget::new(target2, &database),
            WidgetTarget::new(target3, &database),
        ];

        let clipboard = match Clipboard::new() {
            Ok(clipboard) => clipboard,
            Err(e) => panic!("Error fetching clipoard: {e}"),
        };

        MyEguiApp {
            database,
            targets: WidgetTargets::new(targets),
            paths: HashMap::from([("Self".to_string(), Path::new("Self".to_string()))]),
            displayed_path: "Self".to_owned(),
            clipboard,
            position: WidgetTopPosition::default(),
            space_time_position: SpaceTimePosition::default(),
            map: WidgetMap::default(),
            targets_path: HashMap::default(),
            index: 0,

            path_name_io: String::new(),
            global_history_index: 0,
            global_history: Vec::new(),
            global_paths: HashMap::from([("Self".to_string(), Path::new("Self".to_string()))]),
            global_targets: Vec::new(),
            path_selector: String::new(),
            target_selector_poi: String::new(),
            target_selector_container: String::new(),
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

        // add it to history
        self.displayed_path = "Self".to_string();
        self.paths
            .get_mut("Self")
            .unwrap()
            .history
            .push(new_position);
        self.index = self.paths.get_mut("Self").unwrap().history.len() - 1;
    }

    pub fn new_coordinates_from_map(&mut self, point: Option<(f64, f64)>) {
        if let Some((latitude, longitude)) = point {
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
            let time_elapsed =
                (timestamp - *REFERENCE_TIME).num_nanoseconds().unwrap() as f64 / 1e9;

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

            self.paths
                .get_mut(&self.displayed_path)
                .unwrap()
                .history
                .push(new_position);
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

        // Update all NEW path
        for (_, path) in self.global_paths.iter_mut() {
            path.update(
                &self.database,
                self.paths.get("Self").unwrap().history.get(self.index),
            );
        }

        // Update all targets with current position !
        for i in self.targets.targets.iter_mut() {
            i.update(
                &self.database,
                self.paths.get_mut("Self").unwrap().history.get(self.index),
            );
        }

        for i in self.targets_path.values_mut() {
            i.update(
                &self.database,
                self.paths.get("Self").unwrap().history.get(self.index),
                &self.paths,
            )
        }

        // Display targets_path
        for w in self.targets_path.values_mut() {
            w.display(ctx, &self.paths);
        }

        // Display self position
        self.position.display(
            ctx,
            &mut self.database,
            &mut self.index,
            &mut self.displayed_path,
            &mut self.targets,
            &mut self.paths,
        );

        // Display targets
        self.targets.display(
            ctx,
            &mut self.index,
            &mut self.displayed_path,
            &mut self.paths,
            &mut self.targets_path,
        );

        // Display Map
        let point = self
            .map
            .display(ctx, &self.targets, &self.paths, &mut self.targets_path);

        // Add new point from I/O on map
        self.new_coordinates_from_map(point);

        // Display NEW everything
        self.display(ctx);
    }
}
