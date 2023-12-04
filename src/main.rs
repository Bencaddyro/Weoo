// #![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use crate::{
    geolib::Container,
    iolib::{get_space_time_position, load_database},
};
use chrono::prelude::*;
use eframe::egui;
use egui::Color32;
use geolib::{get_current_container, ProcessedPosition, SpaceTimePosition};
use mainlib::{WidgetMap, WidgetTarget, WidgetTargets, WidgetTopPosition};
use once_cell::sync::Lazy;
use std::{collections::HashMap, f64::NAN};
use uuid::Uuid;

mod geolib;
mod guilib;
mod iolib;
mod mainlib;

// Somewhere on Daymar
// Coordinates: x:-18930379393.98 y:-2610297380.75 z:210614.307494
// Coordinates: x:-18930499393.98 y:-2610297380.75 z:210614.307494
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

#[derive(Default)]
struct MyEguiApp {
    // Data
    database: HashMap<String, Container>,
    space_time_position: SpaceTimePosition,

    // App State
    index: usize,
    position_history: Vec<ProcessedPosition>,

    paths: HashMap<String, (Color32, Vec<ProcessedPosition>)>,

    // Gui component
    position: WidgetTopPosition,
    targets: WidgetTargets,
    map: WidgetMap,
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
            .get("Shubin Mining Facility SCD-1")
            .unwrap()
            .to_owned();
        let target2 = database
            .get("Daymar")
            .unwrap()
            .poi
            .get("Eager Flats Aid Shelter")
            .unwrap()
            .to_owned();
        let target3 = database
            .get("Daymar")
            .unwrap()
            .poi
            .get("Kudre Ore")
            .unwrap()
            .to_owned();

        let targets = vec![
            WidgetTarget::new(target1, &database),
            WidgetTarget::new(target2, &database),
            WidgetTarget::new(target3, &database),
        ];

        MyEguiApp {
            database,
            targets: WidgetTargets::new(targets),
            ..Default::default()
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
            altitude = local_coordinates.altitude(&container);
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
        };

        // add it to history
        self.position_history.push(new_position);
        self.index = self.position_history.len() - 1;
    }
}

impl eframe::App for MyEguiApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if let Some(space_time_position_input) = get_space_time_position() {
            if space_time_position_input.coordinates != self.space_time_position.coordinates {
                // New Input !
                self.space_time_position = space_time_position_input;
                self.new_coordinates_input();
            }
        }

        // Update all targets with current position !
        for i in self.targets.targets.iter_mut() {
            i.update(&self.database, self.position_history.get(self.index));
        }

        // Display self position
        self.position.display(
            ctx,
            &mut self.database,
            &mut self.index,
            &mut self.position_history,
            &mut self.targets,
            &mut self.paths,
        );

        // Display targets
        self.targets.display(
            ctx,
            &mut self.index,
            &mut self.position_history,
            &mut self.paths,
        );

        // Display Map
        self.map
            .display(ctx, &mut self.position_history, &self.targets, &self.paths);
    }
}
