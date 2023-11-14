use crate::{
    geolib::{Container, Poi, Vec3d, Vec4d},
    iolib::{get_space_time_position, load_database, SpaceTimePosition},
};
use chrono::prelude::*;
use eframe::egui;
use std::{collections::HashMap, f64::NAN, thread, time};

mod geolib;
mod iolib;

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
    targets: Vec<Poi>,
    new_space_time_position: SpaceTimePosition,
    space_time_position: SpaceTimePosition,
}

impl MyEguiApp {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // Customize egui here with cc.egui_ctx.set_fonts and cc.egui_ctx.set_visuals.
        // Restore app state using cc.storage (requires the "persistence" feature).
        // Use the cc.gl (a glow::Context) to create graphics shaders and buffers that you can use
        // for e.g. egui::PaintCallback.

        let mut new = Self::default();
        new.database = load_database();
        new.reference_time = Utc.with_ymd_and_hms(2020, 1, 1, 0, 0, 0).unwrap();
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

        let target2 = new.database
            .get("Daymar")
            .unwrap()
            .poi
            .get("Eager Flats Aid Shelter")
            .unwrap().to_owned();
        let target3 = new.database
            .get("Daymar")
            .unwrap()
            .poi
            .get("Kudre Ore")
            .unwrap().to_owned();

        new.targets = vec!(target, target2, target3);
        new
    }
}

impl eframe::App for MyEguiApp {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        if let Some(space_time_postion) = get_space_time_position() {
            self.new_space_time_position = space_time_postion;
        } else {
            thread::sleep(time::Duration::from_secs(1));
        }

        if self.new_space_time_position.coordinates != self.space_time_position.coordinates {
            self.space_time_position = self.new_space_time_position.clone();
        }

        let current_pos = &self.space_time_position.coordinates;
        let current_time = self.space_time_position.timestamp;

        // if self.previous_coordinates == current_pos.to_owned() {
        //     return;
        // }

        let current_container = get_current_container(&current_pos, &self.database);
        let time_passed_since_reference_in_seconds: f64 =
            (current_time.timestamp() - self.reference_time.timestamp()) as f64;

        let current_local_pos = current_pos
            .clone()
            .transform_to_local(time_passed_since_reference_in_seconds, &current_container);


        let mut latitute = NAN;
        let mut longitude = NAN;
        let mut altitude = NAN;

        if current_container.name != "None" {
            latitute = current_local_pos.latitude();
            longitude = current_local_pos.longitude();
            altitude = current_local_pos.height(&current_container);
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Self Position");
            ui.label(format!(
                "\
                Timestamp: {current_time},\n\
                Coordinates: x:{} y:{} z:{},\n\
                Container: {},\n\
                Latitute: {latitute},\n\
                Longitude: {longitude},\n\
                Altitude: {altitude},\n\
                ",
                current_pos.x, current_pos.y, current_pos.z,
                current_container.name,
            ));
            ui.separator();
            ui.heading("Target Position");

            ui.spinner();
        });
    }
}

fn mainy() {
    // loop {

    //
    //     println!("{current_pos:?}");
    //     println!("{current_time}");
    //     //New_Player_Global_coordinates
    //
    //
    //
    //     // #-------------------------------------------------player local Long Lat Height--------------------------------------------------

    //
    //     display_target(
    //         &current_pos,
    //         latitute,
    //         longitude,
    //         target,
    //         &database,
    //         time_passed_since_reference_in_seconds,
    //         &current_container,
    //         &current_rotated_pos,
    //     );
    //     display_target(
    //         &current_pos,
    //         latitute,
    //         longitude,
    //         target2,
    //         &database,
    //         time_passed_since_reference_in_seconds,
    //         &current_container,
    //         &current_rotated_pos,
    //     );
    //     display_target(
    //         &current_pos,
    //         latitute,
    //         longitude,
    //         target3,
    //         &database,
    //         time_passed_since_reference_in_seconds,
    //         &current_container,
    //         &current_rotated_pos,
    //     );
    //

    // }
}

fn display_target(
    current_pos: &Vec3d,
    latitude: f64,
    longitude: f64,
    target: &Poi,
    database: &HashMap<String, Container>,
    time_passed_since_reference_in_seconds: f64,
    current_container: &Container,
    current_rotated_pos: &Vec3d,
) {
    let target_container = database.get(&target.container).unwrap();
    // #Grab the rotation speed of the container in the Database and convert it in degrees/s
    let target_rotation_speed_in_hours_per_rotation = target_container.rotation_speed;

    let target_rotation_speed_in_degrees_per_second =
        0.1 * (1.0 / target_rotation_speed_in_hours_per_rotation); //TODO handle divide by 0
                                                                   // #Get the actual rotation state in degrees using the rotation speed of the container, the actual time and a rotational adjustment value
    let target_rotation_state_in_degrees = (target_rotation_speed_in_degrees_per_second
        * time_passed_since_reference_in_seconds
        + target_container.rotation_adjust)
        % 360.0;
    // #get the new player rotated coordinates
    let target_rotated_coordinates = &target
        .coordinates
        .rotate(target_rotation_state_in_degrees.to_radians());

    // #-------------------------------------------------target local Long Lat Height--------------------------------------------------
    //let target_lat_long = get_lat_long_height(&target.pos, &target_container);
    let target_latitude = target.coordinates.latitude();
    let target_longitude = target.coordinates.longitude();
    let target_altitude = target.coordinates.height(target_container);

    // #---------------------------------------------------Distance to target----------------------------------------------------------
    let new_distance_to_target: Vec3d = if current_container.name == target.container {
        target.coordinates.clone() - current_rotated_pos.clone()
    } else {
        target_rotated_coordinates.clone() + target_container.coordinates.clone()
            - current_rotated_pos.clone()
            + current_pos.clone()
    };
    let distance = new_distance_to_target.norm();

    // #----------------------------------------------------------Heading--------------------------------------------------------------

    let bearing_x = target_latitude.to_radians().cos()
        * (target_longitude.to_radians() - longitude.to_radians()).sin();
    let bearing_y = latitude.to_radians().cos() * target_latitude.to_radians().sin()
        - latitude.to_radians().sin()
            * target_latitude.to_radians().cos()
            * (target_longitude.to_radians() - longitude.to_radians()).cos();
    let bearing = (bearing_x.atan2(bearing_y).to_degrees() + 360.0) % 360.0;

    println!(
        "\
        Target: {}\n\
        Latitute: {target_latitude},\n\
        Longitude: {target_longitude},\n\
        Altitude: {target_altitude},\n\
        Distance: {},\n\
        Heading: {},\n\
        --------------------",
        target.name, distance, bearing,
    )
}

fn get_current_container(pos: &Vec3d, database: &HashMap<String, Container>) -> Container {
    let mut current_container = Container {
        name: "None".to_string(),
        coordinates: Vec3d::new(0.0, 0.0, 0.0),
        quaternions: Vec4d::new(0.0, 0.0, 0.0, 0.0),
        marker: false,
        radius_om: 0.0,
        radius_body: 0.0,
        radius_arrival: 0.0,
        time_lines: 0.0,
        rotation_speed: 0.0,
        rotation_adjust: 0.0,
        orbital_radius: 0.0,
        orbital_speed: 0.0,
        orbital_angle: 0.0,
        grid_radius: 0.0,
        poi: HashMap::new(),
    };

    for c in database.values() {
        if (c.coordinates.clone() - pos.clone()).norm() <= 3.0 * c.radius_om {
            current_container = c.clone();
        }
    }
    current_container
}
