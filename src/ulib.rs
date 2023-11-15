use std::collections::HashMap;

use chrono::{DateTime, Utc};

use crate::geolib::{get_current_container, Container, Vec3d};
use crate::iolib::SpaceTimePosition;

#[derive(Default)]
pub struct SelfPosition {
    pub space_time_position: SpaceTimePosition,
    pub elapsed_time_in_seconds: f64,
    pub container: Container,

    pub absolute_coordinates: Vec3d,
    pub local_coordinates: Vec3d,

    pub latitude: f64,
    pub longitude: f64,
    pub altitude: f64,
}

impl SelfPosition {
    pub fn update(
        &mut self,
        space_time_position: &SpaceTimePosition,
        database: &HashMap<String, Container>,
        reference_time: DateTime<Utc>,
    ) {
        self.space_time_position = space_time_position.clone();
        self.absolute_coordinates = space_time_position.coordinates.clone();

        let current_time = self.space_time_position.timestamp;

        self.container = get_current_container(&self.absolute_coordinates, database);

        self.elapsed_time_in_seconds =
            (current_time.timestamp() - reference_time.timestamp()) as f64;

        self.local_coordinates = self
            .absolute_coordinates
            .transform_to_local(self.elapsed_time_in_seconds, &self.container);

        if self.container.name != "None" {
            self.latitude = self.local_coordinates.latitude();
            self.longitude = self.local_coordinates.longitude();
            self.altitude = self.local_coordinates.height(&self.container);
        }
    }

    pub fn display(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            egui::Grid::new("some_unique_id").show(ui, |ui| {
            egui::Grid::new("some_unique_id").show(ui, |ui| {
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
                ui.label(format!("{:.0}°", self.latitude));
                ui.end_row();
                ui.label("Longitude:");
                ui.label(format!("{:.0}°", self.longitude));
                ui.end_row();
                ui.label("Altitude:");
                ui.label(format!("{:.3}km", self.altitude));
                ui.end_row();
            });

ui.add(egui::Separator::default().vertical());
                ui.heading("Placeholder");

                            ui.end_row();

                       });

        });
    }
}
