use crate::geolib::{Container, Poi, SpaceTimePosition, Vec3d, Vec4d};
use arboard::Clipboard;
use chrono::Utc;
use regex::Regex;
use std::collections::HashMap;
use std::fs::{self, File};

fn get_clipboard() -> String {
    let Ok(mut clipboard) = Clipboard::new() else {
        return "".to_string();
    };
    let Ok(content) = clipboard.get_text() else {
        return "".to_string();
    };
    content
}
pub fn get_space_time_position() -> Option<SpaceTimePosition> {
    let s = get_clipboard();
    let timestamp = Utc::now();

    let re = Regex::new(
        r"Coordinates: x:(?<x>-?[0-9]+\.[0-9]+) y:(?<y>-?[0-9]+\.[0-9]+) z:(?<z>-?[0-9]+\.[0-9]+)",
    )
    .unwrap();
    let caps = re.captures(&s)?;
    let coordinates = Vec3d::new(
        caps["x"].parse::<f64>().unwrap() / 1000.0,
        caps["y"].parse::<f64>().unwrap() / 1000.0,
        caps["z"].parse::<f64>().unwrap() / 1000.0,
    );
    Some(SpaceTimePosition {
        coordinates,
        timestamp,
        name: None,
    })
}

pub fn load_database() -> HashMap<String, Container> {
    // Database.json
    let file = fs::File::open("Database.json").expect("file should open read only");
    let json: HashMap<String, HashMap<String, HashMap<String, serde_json::Value>>> =
        serde_json::from_reader(file).expect("file should be proper JSON");

    let mut containers: HashMap<String, Container> = HashMap::new();

    for (_k, v) in json.iter() {
        // println!("keys : {k}");
        for (_kk, vv) in v.iter() {
            // println!("kkeys {kk}");
            let mut poi = HashMap::new();
            let ppoi: HashMap<String, HashMap<String, serde_json::Value>> =
                serde_json::from_value(vv.get("POI").unwrap().to_owned()).unwrap();

            for e in ppoi.into_values() {
                let new_poi = Poi {
                    name: e.get("Name").unwrap().to_string().replace('"', ""),
                    container: e.get("Container").unwrap().to_string().replace('"', ""),
                    coordinates: Vec3d {
                        x: e.get("X").unwrap().as_f64().unwrap(),
                        y: e.get("Y").unwrap().as_f64().unwrap(),
                        z: e.get("Z").unwrap().as_f64().unwrap(),
                    },
                    quaternions: Some(Vec4d {
                        qw: e.get("qw").unwrap().as_f64().unwrap(),
                        qx: e.get("qx").unwrap().as_f64().unwrap(),
                        qy: e.get("qy").unwrap().as_f64().unwrap(),
                        qz: e.get("qz").unwrap().as_f64().unwrap(),
                    }),
                    marker: Some(
                        e.get("QTMarker")
                            .unwrap()
                            .to_string()
                            .replace('"', "")
                            .to_lowercase()
                            .parse()
                            .unwrap(),
                    ),
                };
                poi.insert(new_poi.name.clone(), new_poi);
            }
            let elem = Container {
                name: vv.get("Name").unwrap().to_string().replace('"', ""),
                coordinates: Vec3d::new(
                    vv.get("X").unwrap().as_f64().unwrap(),
                    vv.get("Y").unwrap().as_f64().unwrap(),
                    vv.get("Z").unwrap().as_f64().unwrap(),
                ),
                quaternions: Vec4d::new(
                    vv.get("qw").unwrap().as_f64().unwrap(),
                    vv.get("qx").unwrap().as_f64().unwrap(),
                    vv.get("qy").unwrap().as_f64().unwrap(),
                    vv.get("qz").unwrap().as_f64().unwrap(),
                ),
                marker: vv
                    .get("QTMarker")
                    .unwrap()
                    .to_string()
                    .replace('"', "")
                    .to_lowercase()
                    .parse()
                    .unwrap(),
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
            containers.insert(elem.name.clone(), elem);
        }
    }

    // CustomPoi.json
    if let Ok(file) = File::open("CustomPoi.json") {
        let json: HashMap<String, Poi> =
            serde_json::from_reader(file).expect("file should be proper JSON");

        for poi in json.into_values() {
            if !containers.contains_key(&poi.container) {
                continue;
            }
            containers
                .get_mut(&poi.container)
                .unwrap()
                .poi
                .insert(poi.name.clone(), poi);
        }
    }

    containers
}

pub fn save_history(name: &String, position_history: &Vec<SpaceTimePosition>) {
    let mut file = File::create(format!("{name}.json")).expect("This should work");
    serde_json::to_writer_pretty(&mut file, &position_history)
        .unwrap_or_else(|_| panic!("Fail to write {name}.json"))
}

pub fn import_history(name: &String) -> Vec<SpaceTimePosition> {
    let file = File::open(format!("{name}.json")).unwrap_or_else(|_| panic!("Fail to open {name}.json !"));
    let position_history = serde_json::from_reader(file)
        .unwrap_or_else(|_| panic!("Fail to parse {name}.json, incorrect format"));
    position_history
}
