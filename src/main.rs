use crate::{
    geolib::Container,
    iolib::{get_space_time_position, load_database},
};
use chrono::prelude::*;
use eframe::egui;
use geolib::SpaceTimePosition;
use mainlib::{WidgetPoi, WidgetPosition, WidgetTarget, WidgetTargetSelection};
use std::collections::HashMap;

mod geolib;
mod guilib;
mod iolib;
mod mainlib;
// Coordinates: x:-17068754905.863510 y:-2399480232.5053227 z:-20642.813381

fn main() -> eframe::Result<()> {
    let native_options = eframe::NativeOptions::default();
    eframe::run_native(
        "Weeoo Nav Tool",
        native_options,
        Box::new(|cc| Box::new(MyEguiApp::new(cc))),
    )
}

#[derive(Default)]
struct MyEguiApp {
    database: HashMap<String, Container>,
    reference_time: DateTime<Utc>,
    elapsed_time_in_seconds: f64,

    space_time_position_new: SpaceTimePosition,
    space_time_position: SpaceTimePosition,

    position: WidgetPosition,
    target_selection: WidgetTargetSelection,
    poi_exporter: WidgetPoi,
}

impl MyEguiApp {
    fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        // Customize egui here with cc.egui_ctx.set_fonts and cc.egui_ctx.set_visuals.
        // Restore app state using cc.storage (requires the "persistence" feature).
        // Use the cc.gl (a glow::Context) to create graphics shaders and buffers that you can use
        // for e.g. egui::PaintCallback.

        let database = load_database();
        // Hardcode targets example for Daymar Rally
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
            WidgetTarget {
                target: target1,
                open: true,
            },
            WidgetTarget {
                target: target2,
                open: true,
            },
            WidgetTarget {
                target: target3,
                open: true,
            },
        ];

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
            self.position.update(
                &self.space_time_position,
                &self.database,
                self.elapsed_time_in_seconds,
            );
        }

        self.elapsed_time_in_seconds = (self.space_time_position.timestamp.timestamp()
            - self.reference_time.timestamp()) as f64;

        // Display self position
        self.position.display(ctx);

        // Display targets & target selector
        self.target_selection.display(
            ctx,
            &self.database,
            self.elapsed_time_in_seconds,
            &self.position,
        );

        // Display Poi exporter
        self.poi_exporter
            .display(ctx, &self.position, &self.database);

        self.database = self.poi_exporter.database.clone();
    }
}
