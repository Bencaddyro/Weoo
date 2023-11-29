// #![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use crate::{
    geolib::Container,
    iolib::{get_space_time_position, load_database},
};
use chrono::prelude::*;
use eframe::egui;
use geolib::SpaceTimePosition;
use mainlib::{WidgetMap, WidgetPosition, WidgetTarget, WidgetTargetSelection};
use once_cell::sync::Lazy;
use std::collections::HashMap;

mod geolib;
mod guilib;
mod iolib;
mod mainlib;
// Coordinates: x:-17068754905.863510 y:-2399480232.5053227 z:-20642.813381

// Somewhere on Daymar
// Coordinates: x:-18930379393.98 y:-2610297380.75 z:210614.307494
// Coordinates: x:-18930499393.98 y:-2610297380.75 z:210614.307494
// Coordinates: x:-18930579393.98 y:-2610297380.75 z:210614.307494
// Coordinates: x:-18930679393.98 y:-2610297380.75 z:210614.307494
// Coordinates: x:-18930779393.98 y:-2610297380.75 z:210614.307494
static REFERENCE_TIME: Lazy<DateTime<Utc>> = Lazy::new(|| Utc.with_ymd_and_hms(2020, 1, 1, 0, 0, 0).unwrap());


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
    space_time_position_new: SpaceTimePosition,
    space_time_position: SpaceTimePosition,

    position: WidgetPosition,
    target_selection: WidgetTargetSelection,
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
            map: WidgetMap::new(&database),
            database,
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

            self.position.new_coordinate(self.space_time_position, &self.database);
            self.position.update(&self.database);

            // TODO
            // self.map.new_position(
            //     (self.position.index + 1).to_string(),
            //     &self.positiondinates,
            // );
        }

        // Display self position
        self.position.update(&self.database);
        self.position.display(ctx);

        // Display targets & target selector
        if !self.position.position_history.is_empty() {

        self.target_selection
            .display(ctx, &self.database, &self.position.position_history[self.position.index]);
        };
        // Display Map
        self.map.update();
        self.map.display(ctx, &self.database, &self.position);

        // Update DB from added Poi
        self.database = self.position.database.clone();

        // Update history
        if let Some(i) = self.position.eviction {
            self.position.position_history.remove(i);
            self.position.eviction = None;
        }
        self.position.position_history.append(&mut self.position.addition);

        //TODO
        // if !self.position.position_history.is_empty() & (self.position.position_name != "") {
        //     self.position.position_history[self.position.index].name = self.position.position_name.clone();
        //     self.position.position_name = "".to_string();
        // }

    }
}
