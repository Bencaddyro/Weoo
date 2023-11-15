use crate::{
    geolib::Container,
    iolib::{get_space_time_position, load_database, SpaceTimePosition},
};
use chrono::prelude::*;
use eframe::egui;
use geolib::Poi;
use guilib::WidgetTarget;
use std::collections::HashMap;
use ulib::SelfPosition;

mod geolib;
mod guilib;
mod iolib;
mod ulib;
// Coordinates: x:-17068754905.863510 y:-2399480232.503227 z:-20642.813381

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
    targets: Vec<WidgetTarget>,
    space_time_position_new: SpaceTimePosition,
    space_time_position: SpaceTimePosition,
    space_time_position_old: SpaceTimePosition,
    self_position: SelfPosition,
    target_container: Container,
    target_poi: Poi,
}

impl MyEguiApp {
    fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        // Customize egui here with cc.egui_ctx.set_fonts and cc.egui_ctx.set_visuals.
        // Restore app state using cc.storage (requires the "persistence" feature).
        // Use the cc.gl (a glow::Context) to create graphics shaders and buffers that you can use
        // for e.g. egui::PaintCallback.

        let mut new = MyEguiApp {
            database: load_database(),
            reference_time: Utc.with_ymd_and_hms(2020, 1, 1, 0, 0, 0).unwrap(),
            ..Default::default()
        };
        // TODO
        // Redo a check on how time elapsed is computed

        let target = new
            .database
            .get("Daymar")
            .unwrap()
            .poi
            .get("Shubin Mining Facility SCD-1")
            .unwrap()
            .to_owned();

        let target2 = new
            .database
            .get("Daymar")
            .unwrap()
            .poi
            .get("Eager Flats Aid Shelter")
            .unwrap()
            .to_owned();
        let target3 = new
            .database
            .get("Daymar")
            .unwrap()
            .poi
            .get("Kudre Ore")
            .unwrap()
            .to_owned();

        new.targets = vec![
            WidgetTarget { target, open: true },
            WidgetTarget {
                target: target2,
                open: true,
            },
            WidgetTarget {
                target: target3,
                open: true,
            },
        ];
        new
    }
}

impl eframe::App for MyEguiApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if let Some(space_time_position) = get_space_time_position() {
            self.space_time_position_new = space_time_position;
        }

        if self.space_time_position_new.coordinates != self.space_time_position.coordinates {
            self.space_time_position = self.space_time_position_new.clone();
            self.self_position.update(
                &self.space_time_position,
                &self.database,
                self.reference_time,
            );
        }

        self.self_position.display(ctx);

        // Targets windows

        // Remove hidden targets
        self.targets.retain(|t| t.open);
        // Display others
        for target in &mut self.targets {
            target.display(ctx, &self.database, &self.self_position);
        }

        egui::TopBottomPanel::bottom("bottom_panel")
        .show(ctx, |ui| {
                ui.label("Select Target");

                egui::Grid::new("MainGrid").show(ui, |ui| {

                egui::ComboBox::from_label("Container")
                .selected_text(self.target_container.name.clone())
                .show_ui(ui, |ui| {
                    for container in self.database.values() {
                        ui.selectable_value(&mut self.target_container, container.clone(), container.name.clone());
                    }
                });

                egui::ComboBox::from_label("Poi")
                .selected_text(self.target_poi.name.clone())
                .show_ui(ui, |ui| {
                    for poi in self.target_container.poi.values() {
                        ui.selectable_value(&mut self.target_poi, poi.clone(), poi.name.clone());
                    }
                });

                if ui.button("Add Target").clicked() { self.targets.push(WidgetTarget { open: true, target: self.target_poi.clone() }) };

                ui.end_row();


        });
        });

    }
}
