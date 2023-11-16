use crate::{
    geolib::{Container, Vec3d},
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
        elapsed_time_in_seconds: f64,
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

                if ui.button("Add Target").clicked() {
                    self.targets.push(WidgetTarget {
                        open: true,
                        target: self.target_poi.clone(),
                    })
                };

                ui.end_row();
            });
        });

        // Remove hidden targets
        self.targets.retain(|t| t.open);
        // Display targets windows
        for target in &mut self.targets {
            target.display(ctx, database, elapsed_time_in_seconds, position);
        }
    }
}

impl WidgetPoi {
    pub fn display(
        &mut self,
        ctx: &egui::Context,
        position: &WidgetPosition,
        database: &HashMap<String, Container>,
    ) {
        self.database = database.clone();
        egui::Window::new("Save Poi")
            // .open(&mut self.open)
            .show(ctx, |ui| {
                ui.label("Name:");

                ui.add(egui::TextEdit::singleline(&mut self.name).hint_text("Custom Poi"));
                if ui.button("Save").clicked() {
                    self.save_current_position(position);
                };
            });
    }
}

impl WidgetTarget {
    pub fn display(
        &mut self,
        ctx: &egui::Context,
        database: &HashMap<String, Container>,
        elapsed_time_in_seconds: f64,
        complete_position: &WidgetPosition,
    ) {
        let target_container = database.get(&self.target.container).unwrap();
        // #Grab the rotation speed of the container in the Database and convert it in degrees/s
        let target_rotation_speed_in_hours_per_rotation = target_container.rotation_speed;

        let target_rotation_speed_in_degrees_per_second =
            0.1 * (1.0 / target_rotation_speed_in_hours_per_rotation); //TODO handle divide by 0
                                                                       // #Get the actual rotation state in degrees using the rotation speed of the container, the actual time and a rotational adjustment value
        let target_rotation_state_in_degrees = (target_rotation_speed_in_degrees_per_second
            * elapsed_time_in_seconds
            + target_container.rotation_adjust)
            % 360.0;
        // #get the new player rotated coordinates
        let target_rotated_coordinates = self
            .target
            .coordinates
            .rotate(target_rotation_state_in_degrees.to_radians());

        // #-------------------------------------------------target local Long Lat Height--------------------------------------------------
        let target_latitude = self.target.coordinates.latitude();
        let target_longitude = self.target.coordinates.longitude();
        let target_altitude = self.target.coordinates.height(target_container);

        // #---------------------------------------------------Distance to target----------------------------------------------------------
        let new_distance_to_target: Vec3d =
            if complete_position.container.name == self.target.container {
                self.target.coordinates - complete_position.local_coordinates
            } else {
                target_rotated_coordinates + target_container.coordinates
                    - complete_position.local_coordinates
                    + complete_position.absolute_coordinates
            };
        let distance = new_distance_to_target.norm();

        // #----------------------------------------------------------Heading--------------------------------------------------------------

        let bearing_x = target_latitude.to_radians().cos()
            * (target_longitude.to_radians() - complete_position.longitude.to_radians()).sin();
        let bearing_y = complete_position.latitude.to_radians().cos()
            * target_latitude.to_radians().sin()
            - complete_position.latitude.to_radians().sin()
                * target_latitude.to_radians().cos()
                * (target_longitude.to_radians() - complete_position.longitude.to_radians()).cos();
        let heading = (bearing_x.atan2(bearing_y).to_degrees() + 360.0) % 360.0;

        egui::Window::new(format!("{} - {}", self.target.container, self.target.name))
            .open(&mut self.open)
            .show(ctx, |ui| {
                egui::Grid::new("MainGrid").show(ui, |ui| {
                    ui.label("Latitute:");
                    ui.label(format!("{target_latitude:.0}°"));
                    ui.end_row();
                    ui.label("Longitude:");
                    ui.label(format!("{target_longitude:.0}°"));
                    ui.end_row();
                    ui.label("Altitude:");
                    ui.label(format!("{target_altitude:.3}km"));
                    ui.end_row();
                    ui.label("Distance:");
                    ui.label(format!("{distance:.3}km"));
                    ui.end_row();
                    ui.label("Heading:");
                    ui.label(format!("{heading:.0}°"));
                    ui.end_row();
                });
            });
    }
}
