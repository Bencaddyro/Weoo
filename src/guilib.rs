use crate::{
    geolib::{Container, Poi, Vec3d},
    mainlib::SelfPosition,
};
use std::collections::HashMap;

pub struct WidgetTarget {
    pub open: bool,
    pub target: Poi,
}

impl WidgetTarget {
    pub fn display(
        &mut self,
        ctx: &egui::Context,
        database: &HashMap<String, Container>,
        complete_position: &SelfPosition,
    ) {
        let target_container = database.get(&self.target.container).unwrap();
        // #Grab the rotation speed of the container in the Database and convert it in degrees/s
        let target_rotation_speed_in_hours_per_rotation = target_container.rotation_speed;

        let target_rotation_speed_in_degrees_per_second =
            0.1 * (1.0 / target_rotation_speed_in_hours_per_rotation); //TODO handle divide by 0
                                                                       // #Get the actual rotation state in degrees using the rotation speed of the container, the actual time and a rotational adjustment value
        let target_rotation_state_in_degrees = (target_rotation_speed_in_degrees_per_second
            * complete_position.elapsed_time_in_seconds
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
                self.target.coordinates.clone() - complete_position.local_coordinates.clone()
            } else {
                target_rotated_coordinates.clone() + target_container.coordinates.clone()
                    - complete_position.local_coordinates.clone()
                    + complete_position.absolute_coordinates.clone()
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
