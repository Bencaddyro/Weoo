#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release


use crate::{
    geolib::Container,
    iolib::{get_space_time_position, load_database},
};
use chrono::prelude::*;
use eframe::egui;
use geolib::SpaceTimePosition;
use mainlib::{WidgetMap, WidgetPoi, WidgetPosition, WidgetTarget, WidgetTargetSelection};
use std::collections::HashMap;

mod geolib;
mod guilib;
mod iolib;
mod mainlib;
// Coordinates: x:-17068754905.863510 y:-2399480232.5053227 z:-20642.813381

// Somewhere on Daymar
// Coordinates: x:-18930379393.7 y:-2610297380.75 z:210614.307494


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
    database: HashMap<String, Container>,
    reference_time: DateTime<Utc>,
    time_elapsed: f64,

    space_time_position_new: SpaceTimePosition,
    space_time_position: SpaceTimePosition,

    position: WidgetPosition,
    target_selection: WidgetTargetSelection,
    poi_exporter: WidgetPoi,
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

        let mut targets = HashMap::new();
        targets.insert(
            format!("{} - {}", target1.container, target1.name),
            WidgetTarget::new(target1, &database),
        );
        targets.insert(
            format!("{} - {}", target2.container, target2.name),
            WidgetTarget::new(target2, &database),
        );
        targets.insert(
            format!("{} - {}", target3.container, target3.name),
            WidgetTarget::new(target3, &database),
        );

        MyEguiApp {
            database,
            reference_time: Utc.with_ymd_and_hms(2020, 1, 1, 0, 0, 0).unwrap(),
            position: WidgetPosition::new(),
            target_selection: WidgetTargetSelection::new(targets),
            ..Default::default()
        }
    }
}

impl eframe::App for MyEguiApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if let Some(space_time_position) = get_space_time_position() {
            self.space_time_position_new = space_time_position;
        }

        if self.space_time_position_new.coordinates != self.space_time_position.coordinates {
            self.space_time_position = self.space_time_position_new;
            self.time_elapsed = (self.space_time_position.timestamp - self.reference_time)
                .num_nanoseconds()
                .unwrap() as f64
                / 1e9;

            self.position
                .update(&self.space_time_position, &self.database, self.time_elapsed);
        }

        // Display self position
        self.position.display(ctx);

        // Display targets & target selector
        self.target_selection
            .display(ctx, &self.database, self.time_elapsed, &self.position);

        // Display Poi exporter
        self.poi_exporter.update(&self.database, &self.position);
        self.poi_exporter.display(ctx);

        // Display Map
        self.map.display(ctx);

        // Update DB from added Poi
        self.database = self.poi_exporter.database.clone();
    }
}
