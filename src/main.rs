use arboard::Clipboard;
use serde::{Deserialize, Serialize};
use std::fs;
use serde_json;
use regex::Regex;
use std::collections::HashMap;
use chrono::prelude::*;
use std::{thread, time};


// TODO
// #Sets some variables
// Reference_time_UTC = datetime.datetime(2020, 1, 1)
// Epoch = datetime.datetime(1970, 1, 1)
// Reference_time = (Reference_time_UTC - Epoch).total_seconds()
//

fn main() {
    println!("Hello, world!");
    let database: HashMap<String,Container> = load_database();

    let reference_time = Utc.with_ymd_and_hms(2020, 1, 1, 0, 0, 0).unwrap();
    let mut current_time;

    let mut old_pos = Vec3d{x:0.0,y:0.0,z:0.0};


    let target = database.get("Daymar").unwrap().poi.get("Shubin Mining Facility SCD-1").unwrap();

    let target2 = database.get("Daymar").unwrap().poi.get("Eager Flats Aid Shelter").unwrap();
    let target3 = database.get("Daymar").unwrap().poi.get("Kudre Ore").unwrap();


    loop {
        let Some(current_pos) = get_pos() else { thread::sleep(time::Duration::from_secs(1)); continue };
        current_time = Utc::now();

        if old_pos == current_pos { continue };
        old_pos = current_pos.clone();

        println!("{current_pos:?}");
        println!("{current_time}");
        //New_Player_Global_coordinates

        let time_passed_since_reference_in_seconds: f64 = (current_time.timestamp() - reference_time.timestamp()) as f64;


        let current_container = get_current_container(&current_pos, &database);


        let current_rotated_pos= get_local_rotated_coordinates(&time_passed_since_reference_in_seconds, &current_pos, &current_container );
        // New_player_local_rotated_coordinates





        // #-------------------------------------------------player local Long Lat Height--------------------------------------------------
        let mut current_lat_long = Vec3d{x:0.0,y:0.0,z:0.0};
        if current_container.name != "None" {
            current_lat_long = get_lat_long_height(&current_rotated_pos, &current_container);
        }


        display_target(&current_pos, &current_lat_long, &target, &database, time_passed_since_reference_in_seconds, &current_container, &current_rotated_pos);
        display_target(&current_pos, &current_lat_long, &target2, &database, time_passed_since_reference_in_seconds, &current_container, &current_rotated_pos);
        display_target(&current_pos, &current_lat_long, &target3, &database, time_passed_since_reference_in_seconds, &current_container, &current_rotated_pos);




                // #----------------------------------------------------Course Deviation to POI--------------------------------------------------------
                // #get the vector between current_pos and previous_pos
                // Previous_current_pos_vector = {}
                // for i in ['X', 'Y', 'Z']:
                //     Previous_current_pos_vector[i] = New_player_local_rotated_coordinates[i] - Old_player_local_rotated_coordinates[i]
                //
                //
                // #get the vector between current_pos and target_pos
                // Current_target_pos_vector = {}
                // for i in ['X', 'Y', 'Z']:
                //     Current_target_pos_vector[i] = Target[i] - New_player_local_rotated_coordinates[i]
                //
                //
                // #get the angle between the current-target_pos vector and the previous-current_pos vector
                // Total_deviation_from_target = angle_between_vectors(Previous_current_pos_vector, Current_target_pos_vector)


                // if Total_deviation_from_target <= 10:
                //     Total_deviation_from_target_color = "#00ff00"
                // elif Total_deviation_from_target <= 20:
                //     Total_deviation_from_target_color = "#ffd000"
                // else:
                //     Total_deviation_from_target_color = "#ff3700"
                //
                //
                // #----------------------------------------------------------Flat_angle--------------------------------------------------------------
                // previous = Old_player_local_rotated_coordinates
                // current = New_player_local_rotated_coordinates


                // #Vector AB (Previous -> Current)
                // previous_to_current = {}
                // for i in ["X", "Y", "Z"]:
                //     previous_to_current[i] = current[i] - previous[i]
                //
                // #Vector AC (C = center of the planet, Previous -> Center)
                // previous_to_center = {}
                // for i in ["X", "Y", "Z"]:
                //     previous_to_center[i] = 0 - previous[i]
                //
                // #Vector BD (Current -> Target)
                // current_to_target = {}
                // for i in ["X", "Y", "Z"]:
                //     current_to_target[i] = Target[i] - current[i]
                //
                //     #Vector BC (C = center of the planet, Current -> Center)
                // current_to_center = {}
                // for i in ["X", "Y", "Z"]:
                //     current_to_center[i] = 0 - current[i]



                // #Normal vector of a plane:
                // #abc : Previous/Current/Center
                // n1 = {}
                // n1["X"] = previous_to_current["Y"] * previous_to_center["Z"] - previous_to_current["Z"] * previous_to_center["Y"]
                // n1["Y"] = previous_to_current["Z"] * previous_to_center["X"] - previous_to_current["X"] * previous_to_center["Z"]
                // n1["Z"] = previous_to_current["X"] * previous_to_center["Y"] - previous_to_current["Y"] * previous_to_center["X"]
                //
                // #acd : Previous/Center/Target
                // n2 = {}
                // n2["X"] = current_to_target["Y"] * current_to_center["Z"] - current_to_target["Z"] * current_to_center["Y"]
                // n2["Y"] = current_to_target["Z"] * current_to_center["X"] - current_to_target["X"] * current_to_center["Z"]
                // n2["Z"] = current_to_target["X"] * current_to_center["Y"] - current_to_target["Y"] * current_to_center["X"]

                // Flat_angle = angle_between_vectors(n1, n2)
                //
                //
                // if Flat_angle <= 10:
                //     Flat_angle_color = "#00ff00"
                // elif Flat_angle <= 20:
                //     Flat_angle_color = "#ffd000"
                // else:
                //     Flat_angle_color = "#ff3700"




        println!("\
            --------------------\n\
            Self Position\n\
            Container: {},\n\
            Latitute: {},\n\
            Longitude: {},\n\
            Altitude: {},\n\
            --------------------",
            current_container.name,
            current_lat_long.x,
            current_lat_long.y,
            current_lat_long.z,
        )
    }
}


fn display_target(current_pos: &Vec3d, current_lat_long: &Vec3d, target: &Poi, database: &HashMap<String,Container>, time_passed_since_reference_in_seconds: f64, current_container: &Container, current_rotated_pos: &Vec3d){
    let target_container = database.get(&target.container).unwrap();
    // #Grab the rotation speed of the container in the Database and convert it in degrees/s
    let target_rotation_speed_in_hours_per_rotation = target_container.rotation_speed;

    let target_rotation_speed_in_degrees_per_second = 0.1 * (1.0/target_rotation_speed_in_hours_per_rotation); //TODO handle divide by 0
    // #Get the actual rotation state in degrees using the rotation speed of the container, the actual time and a rotational adjustment value
    let target_rotation_state_in_degrees = ( target_rotation_speed_in_degrees_per_second * time_passed_since_reference_in_seconds + target_container.rotation_adjust ) % 360.0;
    // #get the new player rotated coordinates
    let target_rotated_coordinates = rotate_point_2_d( &target.pos, target_rotation_state_in_degrees.to_radians() );

    // #-------------------------------------------------target local Long Lat Height--------------------------------------------------
    let target_lat_long = get_lat_long_height(&target.pos, &target_container);

    // #---------------------------------------------------Distance to target----------------------------------------------------------
        let new_distance_to_target: Vec3d;
        if current_container.name == target.container {
            let x = target.pos.x - current_rotated_pos.x;
            let y = target.pos.y - current_rotated_pos.y;
            let z = target.pos.z - current_rotated_pos.z;
            new_distance_to_target = Vec3d{x,y,z};
        } else {
            let x = target_rotated_coordinates.x + target_container.x - current_rotated_pos.x + current_pos.x;
            let y = target_rotated_coordinates.y + target_container.y - current_rotated_pos.y + current_pos.y;
            let z = target_rotated_coordinates.z + target_container.z - current_rotated_pos.z + current_pos.z;
            new_distance_to_target = Vec3d{x,y,z};
        }
        let distance = vector_norm(&new_distance_to_target);

        // #----------------------------------------------------------Heading--------------------------------------------------------------

        let bearing_x = target_lat_long.x.to_radians().cos() * (target_lat_long.y.to_radians() - current_lat_long.y.to_radians()).sin();
        let bearing_y = current_lat_long.x.to_radians().cos() * target_lat_long.x.to_radians().sin() - current_lat_long.x.to_radians().sin() * target_lat_long.x.to_radians().cos() * (target_lat_long.y.to_radians() - current_lat_long.y.to_radians()).cos();
        let bearing = (bearing_x.atan2(bearing_y).to_degrees() + 360.0) % 360.0;


    println!("\
        Target: {}\n\
        Latitute: {},\n\
        Longitude: {},\n\
        Altitude: {},\n\
        Distance: {},\n\
        Heading: {},\n\
        --------------------",
        target.name,
        target_lat_long.x,
        target_lat_long.y,
        target_lat_long.z,
        distance,
        bearing,
    )
}


fn get_clipboard() -> String {
    let Ok(mut clipboard) = Clipboard::new() else { return "".to_string() };
    let Ok(content) = clipboard.get_text() else { return "".to_string() };
    // println!("Clipboard text was: {content}");
    content
}

fn get_pos() -> Option<Vec3d> {
    let s = get_clipboard();
    let re = Regex::new(r"Coordinates: x:(?<x>-?[0-9]+\.[0-9]+) y:(?<y>-?[0-9]+\.[0-9]+) z:(?<z>-?[0-9]+\.[0-9]+)").unwrap();
    let Some(caps) = re.captures(&s) else { return None };
    // println!("{caps:?}");
    let x = caps["x"].parse::<f64>().unwrap() / 1000.0;
    let y = caps["y"].parse::<f64>().unwrap() / 1000.0;
    let z = caps["z"].parse::<f64>().unwrap() / 1000.0;
    Some(Vec3d{x,y,z})
}
//Coordinates: x:22462016306.0103 y:37185625645.8346 z:0.0

fn load_database() -> HashMap<String,Container> {
    let file = fs::File::open("Database.json").expect("file should open read only");
    let json: HashMap<String, HashMap<String, HashMap<String, serde_json::Value>>> = serde_json::from_reader(file).expect("file should be proper JSON");

    let mut containers: HashMap<String,Container> = HashMap::new();

    for (_k,v) in json.iter() {
        // println!("keys : {k}");
        for (_kk,vv) in v.iter() {
            // println!("kkeys {kk}");
            let mut poi = HashMap::new();
            let ppoi: HashMap<String, HashMap<String, serde_json::Value>> = serde_json::from_value(vv.get("POI").unwrap().to_owned()).unwrap();

            for e in ppoi.into_values() {
                let new_poi = Poi{
                    name: e.get("Name").unwrap().to_string().replace('"', ""),
                    container: e.get("Container").unwrap().to_string().replace('"', ""),
                    pos: Vec3d{
                        x: e.get("X").unwrap().as_f64().unwrap(),
                        y: e.get("Y").unwrap().as_f64().unwrap(),
                        z: e.get("Z").unwrap().as_f64().unwrap(),
                    },
                    qw: e.get("qw").unwrap().as_f64().unwrap(),
                    qx: e.get("qx").unwrap().as_f64().unwrap(),
                    qy: e.get("qy").unwrap().as_f64().unwrap(),
                    qz: e.get("qz").unwrap().as_f64().unwrap(),
                    marker: e.get("QTMarker").unwrap().to_string().replace('"', "").to_lowercase().parse().unwrap(),
                };
                poi.insert(new_poi.name.clone(),new_poi);


            };
            let elem = Container{
                name: vv.get("Name").unwrap().to_string().replace('"', ""),
                x: vv.get("X").unwrap().as_f64().unwrap(),
                y: vv.get("Y").unwrap().as_f64().unwrap(),
                z: vv.get("Z").unwrap().as_f64().unwrap(),
                qw: vv.get("qw").unwrap().as_f64().unwrap(),
                qx: vv.get("qx").unwrap().as_f64().unwrap(),
                qy: vv.get("qy").unwrap().as_f64().unwrap(),
                qz: vv.get("qz").unwrap().as_f64().unwrap(),
                marker: vv.get("QTMarker").unwrap().to_string().replace('"', "").to_lowercase().parse().unwrap(),
                radius_om: vv.get("OM Radius").unwrap().as_f64().unwrap(),
                radius_body: vv.get("Body Radius").unwrap().as_f64().unwrap(),
                radius_arrival: vv.get("Arrival Radius").unwrap().as_f64().unwrap(),
                time_lines: vv.get("Time Lines").unwrap().as_f64().unwrap(),
                rotation_speed: vv.get("Rotation Speed").unwrap().as_f64().unwrap(),
                rotation_adjust: vv.get("Rotation Adjust").unwrap().as_f64().unwrap(),
                orbital_radius: vv.get("Orbital Radius").unwrap().as_f64().unwrap(),
                orbital_speed: vv.get("Orbital Speed").unwrap().as_f64().unwrap(),
                orbital_angle: vv.get("Orbital Angle").unwrap().as_f64().unwrap(),
                grid_radius: vv.get("Grid Radius").unwrap().as_f64().unwrap(),
                poi,
            };
            containers.insert(elem.name.clone(),elem);
        }
    };
    // println!("blah!");
    // println!("{containers:?}");
    containers
}

#[derive(Debug, Deserialize, Serialize, Clone)]
struct Container {
    name: String,
    x: f64,
    y: f64,
    z: f64,
    qw: f64,
    qx: f64,
    qy: f64,
    qz: f64,
    marker: bool,
    radius_om: f64,
    radius_body: f64,
    radius_arrival: f64,
    time_lines: f64,
    rotation_speed: f64,
    rotation_adjust: f64,
    orbital_radius: f64,
    orbital_speed: f64,
    orbital_angle: f64,
    grid_radius: f64,
    poi: HashMap<String,Poi>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
struct Poi {
    name: String,
    container: String,
    pos: Vec3d,
    qw: f64,
    qx: f64,
    qy: f64,
    qz: f64,
    marker: bool,
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
struct Vec3d {
    x: f64,
    y: f64,
    z: f64,
}

fn vector_norm(p: &Vec3d) -> f64 {
    return (p.x*p.x+p.y*p.y+p.z*p.z).sqrt()
}

fn vector_product(a: &Vec3d, b: &Vec3d) -> f64 {
    return a.x*b.x+a.y*b.y+a.z*b.z
}

fn angle_between_vectors(a: &Vec3d, b: &Vec3d) -> f64 {
    let angle = (vector_product(a, b) / (vector_norm(a) + vector_norm(b))).acos().to_degrees();
    if angle.is_nan() { // ZeroDivision
        return 0.0;
    }
    angle
}

fn rotate_point_2_d(p: &Vec3d, angle: f64) -> Vec3d {
    let x = angle.cos() * p.x - angle.sin() * p.y;
    let y = angle.sin() * p.x + angle.cos() * p.y;
    let z = p.z;
    Vec3d{x, y, z}
}

fn get_current_container(pos: &Vec3d, database: &HashMap<String,Container>) -> Container {

    let mut current_container = Container {
        name: "None".to_string(),
        x: 0.0,
        y: 0.0,
        z: 0.0,
        qw: 0.0,
        qx: 0.0,
        qy: 0.0,
        qz: 0.0,
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
        let x = c.x-pos.x;
        let y = c.y-pos.y;
        let z = c.z-pos.z;
        let v = Vec3d{x,y,z};
        if vector_norm(&v) <= 3.0 * c.radius_om {

            current_container = c.clone();
        }
    }
    current_container
}


fn get_local_rotated_coordinates(time_elapsed: &f64, pos: &Vec3d, container: &Container) -> Vec3d {
    let mut rotation_speed_in_degrees_per_second = 0.1 * ( 1.0 / container.rotation_speed);
    if rotation_speed_in_degrees_per_second.is_nan() {
        rotation_speed_in_degrees_per_second = 0.0;
    };

    let rotation_state_in_degrees = (rotation_speed_in_degrees_per_second * time_elapsed + container.rotation_adjust) % 360.0;
    let local_unrotated_coordinates = Vec3d {
            x: pos.x-container.x,
            y: pos.y-container.y,
            z: pos.z-container.z,
    };
    let local_rotated_coordinates = rotate_point_2_d(&local_unrotated_coordinates, (-rotation_state_in_degrees).to_radians());
    return local_rotated_coordinates;
}

fn get_lat_long_height(pos: &Vec3d, container: &Container) -> Vec3d {
    let radius = container.radius_body;
    let radial_distance = vector_norm(&pos);
    let height = radial_distance - radius;

    let mut latitude = (pos.z / radial_distance).asin().to_degrees();
    if latitude.is_nan() {
        latitude = 0.0;
    };
    let mut longitude = pos.x.atan2(pos.y).to_degrees() * -1.0;
    if longitude.is_nan() {
        longitude = 0.0;
    }

    return Vec3d { x: latitude, y: longitude, z:  height};
}

// def get_closest_POI(X : float, Y : float, Z : float, Container : dict, Quantum_marker : bool = False):
//
//     Distances_to_POIs = []
//
//     for POI in Container["POI"]:
//         Vector_POI = {
//             "X": abs(X - Container["POI"][POI]["X"]),
//             "Y": abs(Y - Container["POI"][POI]["Y"]),
//             "Z": abs(Z - Container["POI"][POI]["Z"])
//         }
//
//         Distance_POI = vector_norm(Vector_POI)
//
//         if Quantum_marker and Container["POI"][POI]["QTMarker"] == "TRUE" or not Quantum_marker:
//             Distances_to_POIs.append({"Name" : POI, "Distance" : Distance_POI})
//
//     Target_to_POIs_Distances_Sorted = sorted(Distances_to_POIs, key=lambda k: k['Distance'])
//     return Target_to_POIs_Distances_Sorted
//
//
//
// def get_closest_oms(X : float, Y : float, Z : float, Container : dict):
//     Closest_OM = {}
//
//     if X >= 0:
//         Closest_OM["X"] = {"OM" : Container["POI"]["OM-5"], "Distance" : vector_norm({"X" : X - Container["POI"]["OM-5"]["X"], "Y" : Y - Container["POI"]["OM-5"]["Y"], "Z" : Z - Container["POI"]["OM-5"]["Z"]})}
//     else:
//         Closest_OM["X"] = {"OM" : Container["POI"]["OM-6"], "Distance" : vector_norm({"X" : X - Container["POI"]["OM-6"]["X"], "Y" : Y - Container["POI"]["OM-6"]["Y"], "Z" : Z - Container["POI"]["OM-6"]["Z"]})}
//     if Y >= 0:
//         Closest_OM["Y"] = {"OM" : Container["POI"]["OM-3"], "Distance" : vector_norm({"X" : X - Container["POI"]["OM-3"]["X"], "Y" : Y - Container["POI"]["OM-3"]["Y"], "Z" : Z - Container["POI"]["OM-3"]["Z"]})}
//     else:
//         Closest_OM["Y"] = {"OM" : Container["POI"]["OM-4"], "Distance" : vector_norm({"X" : X - Container["POI"]["OM-4"]["X"], "Y" : Y - Container["POI"]["OM-4"]["Y"], "Z" : Z - Container["POI"]["OM-4"]["Z"]})}
//     if Z >= 0:
//         Closest_OM["Z"] = {"OM" : Container["POI"]["OM-1"], "Distance" : vector_norm({"X" : X - Container["POI"]["OM-1"]["X"], "Y" : Y - Container["POI"]["OM-1"]["Y"], "Z" : Z - Container["POI"]["OM-1"]["Z"]})}
//     else:
//         Closest_OM["Z"] = {"OM" : Container["POI"]["OM-2"], "Distance" : vector_norm({"X" : X - Container["POI"]["OM-2"]["X"], "Y" : Y - Container["POI"]["OM-2"]["Y"], "Z" : Z - Container["POI"]["OM-2"]["Z"]})}
//
//     return Closest_OM
//

