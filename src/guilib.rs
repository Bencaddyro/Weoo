use crate::{
    geolib::Container,
    mainlib::{WidgetPoi, WidgetPosition, WidgetTarget, WidgetTargetSelection},
};
use std::collections::HashMap;

impl WidgetPosition {
    pub fn display(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            egui::Grid::new("MainTopGrid").show(ui, |ui| {
                egui::Grid::new("SelfPosition").show(ui, |ui| {
                    ui.heading("Self Position");
                    ui.spinner();
                    ui.end_row();

                    ui.label("Timestamp:");
                    ui.label(format!("{}", self.space_time_position.timestamp));
                    ui.end_row();
                    ui.label("Coordinates:");
                    ui.label(format!(
                        "x:{} y:{} z:{}",
                        self.space_time_position.coordinates.x,
                        self.space_time_position.coordinates.y,
                        self.space_time_position.coordinates.z
                    ));
                    ui.end_row();
                    ui.label("Container:");
                    ui.label(self.container.name.to_string());
                    ui.end_row();

                    ui.label("Latitute:");
                    ui.label(format!("{:.2}°", self.latitude));
                    ui.end_row();
                    ui.label("Longitude:");
                    ui.label(format!("{:.2}°", self.longitude));
                    ui.end_row();
                    ui.label("Altitude:");
                    ui.label(format!("{:.3}km", self.altitude));
                    ui.end_row();
                });

                ui.add(egui::Separator::default().vertical());

                ui.end_row();
            });
        });
    }
}

impl WidgetTargetSelection {
    pub fn display(
        &mut self,
        ctx: &egui::Context,
        database: &HashMap<String, Container>,
        elapsed_time: f64,
        position: &WidgetPosition,
    ) {
        egui::TopBottomPanel::bottom("bottom_panel").show(ctx, |ui| {
            ui.label("Select Target");

            egui::Grid::new("MainGrid").show(ui, |ui| {
                egui::ComboBox::from_label("Container")
                    .selected_text(self.target_container.name.clone())
                    .show_ui(ui, |ui| {
                        for container in database.values() {
                            ui.selectable_value(
                                &mut self.target_container,
                                container.clone(),
                                container.name.clone(),
                            );
                        }
                    });

                egui::ComboBox::from_label("Poi")
                    .selected_text(self.target_poi.name.clone())
                    .show_ui(ui, |ui| {
                        for poi in database
                            .get(&self.target_container.name)
                            .unwrap()
                            .poi
                            .values()
                        {
                            ui.selectable_value(
                                &mut self.target_poi,
                                poi.clone(),
                                poi.name.clone(),
                            );
                        }
                    });

                if ui.button("Add Target").clicked()
                    & database.contains_key(&self.target_poi.container)
                {
                    self.targets.insert(
                        format!("{} - {}", self.target_container.name, self.target_poi.name),
                        WidgetTarget::new(self.target_poi.clone(), database),
                    );
                };

                ui.end_row();
            });
        });

        // Remove hidden targets
        self.targets.retain(|_, v| v.open);
        // Display targets windows
        for target in &mut self.targets.values_mut() {
            target.update(database, elapsed_time, position);
            target.display(ctx);
        }
    }
}

impl WidgetTarget {
    pub fn display(&mut self, ctx: &egui::Context) {
        egui::Window::new(format!("{} - {}", self.target.container, self.target.name))
            .open(&mut self.open)
            .show(ctx, |ui| {
                egui::Grid::new("MainGrid").show(ui, |ui| {
                    ui.label("Latitute:");
                    ui.label(format!("{:.2}°", self.latitude));
                    ui.end_row();
                    ui.label("Longitude:");
                    ui.label(format!("{:.2}°", self.longitude));
                    ui.end_row();
                    ui.label("Altitude:");
                    ui.label(format!("{:.3}km", self.altitude));
                    ui.end_row();
                    ui.label("Distance:");
                    ui.label(format!("{:.3}km", self.distance));
                    ui.end_row();
                    ui.label("Heading:");
                    ui.label(format!("{:.2}°", self.heading));
                    ui.end_row();
                    // ui.label("Delta:");
                    // ui.label(format!("{:?}", self.delta_distance));
                    // ui.end_row();
                });
            });
    }
}

impl WidgetPoi {
    pub fn display(&mut self, ctx: &egui::Context) {
        egui::Window::new("Save Poi")
            // .open(&mut self.open)
            .show(ctx, |ui| {
                ui.label("Name:");

                ui.add(egui::TextEdit::singleline(&mut self.name).hint_text("Custom Poi"));
                if ui.button("Save").clicked() {
                    self.save_current_position();
                };
            });
    }
}
