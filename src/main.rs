use crate::{
    geolib::{Container, Poi, Vec3d, Vec4d},
    iolib::{get_space_time_position, load_database},
};
use chrono::prelude::*;
use std::{collections::HashMap, f64::NAN};
use std::{thread, time};

mod geolib;
mod iolib;

// TODO
// #Sets some variables
// Reference_time_UTC = datetime.datetime(2020, 1, 1)
// Epoch = datetime.datetime(1970, 1, 1)
// Reference_time = (Reference_time_UTC - Epoch).total_seconds()
//

fn main() {
    println!("Hello, world!");
    let database: HashMap<String, Container> = load_database();

    let reference_time = Utc.with_ymd_and_hms(2020, 1, 1, 0, 0, 0).unwrap();

    let mut old_pos = Vec3d {
        x: 0.0,
        y: 0.0,
        z: 0.0,
    };

    let target = database
        .get("Daymar")
        .unwrap()
        .poi
        .get("Shubin Mining Facility SCD-1")
        .unwrap();

    let target2 = database
        .get("Daymar")
        .unwrap()
        .poi
        .get("Eager Flats Aid Shelter")
        .unwrap();
    let target3 = database
        .get("Daymar")
        .unwrap()
        .poi
        .get("Kudre Ore")
        .unwrap();

    loop {
        let Some(space_time_postion) = get_space_time_position() else {
            thread::sleep(time::Duration::from_secs(1));
            continue;
        };
        let current_pos = space_time_postion.coordinates;
        let current_time = space_time_postion.timestamp;

        if old_pos == current_pos {
            continue;
        };
        old_pos = current_pos.clone();

        println!("{current_pos:?}");
        println!("{current_time}");
        //New_Player_Global_coordinates

        let time_passed_since_reference_in_seconds: f64 =
            (current_time.timestamp() - reference_time.timestamp()) as f64;

        let current_container = get_current_container(&current_pos, &database);

        let current_rotated_pos = current_pos
            .clone()
            .transform_to_local(time_passed_since_reference_in_seconds, &current_container);
        // New_player_local_rotated_coordinates

        // #-------------------------------------------------player local Long Lat Height--------------------------------------------------
        let mut latitute = NAN;
        let mut longitude = NAN;
        let mut altitude = NAN;

        if current_container.name != "None" {
            latitute = current_pos.latitude();
            longitude = current_pos.longitude();
            altitude = current_pos.height(&current_container);
        }

        display_target(
            &current_pos,
            latitute,
            longitude,
            target,
            &database,
            time_passed_since_reference_in_seconds,
            &current_container,
            &current_rotated_pos,
        );
        display_target(
            &current_pos,
            latitute,
            longitude,
            target2,
            &database,
            time_passed_since_reference_in_seconds,
            &current_container,
            &current_rotated_pos,
        );
        display_target(
            &current_pos,
            latitute,
            longitude,
            target3,
            &database,
            time_passed_since_reference_in_seconds,
            &current_container,
            &current_rotated_pos,
        );

        println!(
            "\
            --------------------\n\
            Self Position\n\
            Container: {},\n\
            Latitute: {latitute},\n\
            Longitude: {longitude},\n\
            Altitude: {altitude},\n\
            --------------------",
            current_container.name,
        )
    }
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
