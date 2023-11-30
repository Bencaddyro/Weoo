use crate::geolib::{Container, Poi, ProcessedPosition, Vec3d};
use std::{collections::HashMap, f64::consts::PI};

#[derive(Clone, Default)]
pub struct WidgetTopPosition {
    // pub position_history: Vec<ProcessedPosition>,
    // pub index: usize,
    pub addition: Vec<ProcessedPosition>,
    pub eviction: Option<usize>,

    // pub position: ProcessedPosition,

    // POI exporter
    pub database: HashMap<String, Container>,
    pub name: String,
    // Coordinates: x:-18930779393.98 y:-2610297380.75 z:210614.307494

    // History
    pub history_name: String,

    // Target selector
    pub target_container: Container,
    pub target_poi: Poi,
}

#[derive(Default)]
pub struct WidgetTargets {
    // pub target_container: Container,
    // pub target_poi: Poi,
    pub targets: Vec<WidgetTarget>,
}

#[derive(Default)]
pub struct WidgetTarget {
    pub open: bool,
    pub target: Poi,
    pub latitude: f64,
    pub longitude: f64,
    pub altitude: f64,
    pub distance: f64,
    pub heading: f64,
    pub delta_distance: Vec3d,
}

#[derive(Default)]
pub struct WidgetMap {
    // pub open: bool,
    // pub targets: Vec<(String, [f64; 2])>,
    // pub target_container: Container,
    // pub target_poi: Poi,
    // pub travel: Vec<(String, [f64; 2])>,
    // pub eviction: Vec<usize>,
    // pub eviction_self: Vec<usize>,
}

impl WidgetTopPosition {
    pub fn new() -> Self {
        Self {
            // latitude: NAN,
            // longitude: NAN,
            // altitude: NAN,
            ..Default::default()
        }
    }

    pub fn update(&mut self, database: &HashMap<String, Container>) {
        self.database = database.clone();

        // self.space_time_position = *space_time_position;
        // if self.position_history.is_empty() {
        //     return;
        // };
        //
        // // self.timestamp = self.position_history[self.index].space_time_position.timestamp;
        //
        // self.position_history[self.index]
        //     .space_time_position
        //     .coordinates;
    }

    // TODO move r/w files to iolib
    // pub fn save_current_position(&mut self) {
    //     let mut custom_pois: HashMap<String, Poi>;
    //     // Open Custom Poi file
    //     if let Ok(file) = fs::File::open("CustomPoi.json") {
    //         custom_pois = serde_json::from_reader(file).expect("file should be proper JSON");
    //     } else {
    //         println!("No file");
    //         custom_pois = HashMap::new();
    //     };
    //
    //     // Search for existing Poi with this name
    //     if custom_pois.contains_key(&self.name) {
    //         println!("Poi already exist, default override")
    //     }
    //
    //     let new_poi = if (self.position.container.name == "Space")
    //         | (self.position.container.name.is_empty())
    //     {
    //         Poi {
    //             name: self.name.clone(),
    //             container: "Space".to_string(),
    //             coordinates: self.position
    //                 .space_time_position
    //                 .coordinates,
    //             quaternions: None,
    //             marker: None,
    //         }
    //     } else {
    //         Poi {
    //             name: self.name.clone(),
    //             container: self.position.container.name.clone(),
    //             coordinates: self.position.local_coordinates,
    //             quaternions: None,
    //             marker: None,
    //         }
    //     };
    //     // Add to set
    //     custom_pois.insert(self.name.clone(), new_poi.clone());
    //
    //     // Add to database
    //     self.database
    //         .get_mut(&new_poi.container)
    //         .unwrap()
    //         .poi
    //         .insert(self.name.clone(), new_poi);
    //
    //     // Write files
    //     let mut file = std::fs::File::create("CustomPoi.json").expect("This should work");
    //     serde_json::to_writer_pretty(&mut file, &custom_pois)
    //         .expect("Fail to write cutom poi json");
    // }
}

impl WidgetTargets {
    pub fn new(targets: Vec<WidgetTarget>) -> Self {
        Self { targets }
    }
}

impl WidgetTarget {
    pub fn new(target: Poi, database: &HashMap<String, Container>) -> Self {
        Self {
            open: true,
            latitude: target.coordinates.latitude(),
            longitude: target.coordinates.longitude(),
            altitude: target
                .coordinates
                .altitude(database.get(&target.container).unwrap_or_else(|| {
                    panic!("No Container with that name : \"{}\"", &target.container)
                })),

            target,
            ..Default::default()
        }
    }

    pub fn update(
        &mut self,
        database: &HashMap<String, Container>,
        complete_position: Option<&ProcessedPosition>,
    ) {
        if let Some(complete_position) = complete_position {
            let target_container = database.get(&self.target.container).unwrap();
            // #Grab the rotation speed of the container in the Database and convert it in degrees/s
            let target_rotation_speed_in_hours_per_rotation = target_container.rotation_speed;

            let target_rotation_speed_in_degrees_per_second =
                0.1 * (1.0 / target_rotation_speed_in_hours_per_rotation); //TODO handle divide by 0
                                                                           // #Get the actual rotation state in degrees using the rotation speed of the container, the actual time and a rotational adjustment value
            let target_rotation_state_in_degrees = (target_rotation_speed_in_degrees_per_second
                * complete_position.time_elapsed
                + target_container.rotation_adjust)
                % 360.0;

            // Target rotated coordinates (still relative to container center)
            let target_rotated_coordinates = self
                .target
                .coordinates
                .rotate(target_rotation_state_in_degrees.to_radians());

            // #---------------------------------------------------Distance to target----------------------------------------------------------
            self.delta_distance = if complete_position.container.name == self.target.container {
                self.target.coordinates - complete_position.local_coordinates
            } else {
                target_rotated_coordinates + target_container.coordinates
                    // - complete_position.local_coordinates // why this ?
                    // + complete_position.absolute_coordinates // and why a + ?
                    - complete_position.space_time_position.coordinates
            };
            self.distance = self.delta_distance.norm();

            // #----------------------------------------------------------Heading--------------------------------------------------------------
            // If planetary !
            self.heading = (complete_position
                .space_time_position
                .coordinates
                .loxodromie_to(self.target.coordinates)
                + 2.0 * PI)
                % (2.0 * PI);
        }
    }
}

impl WidgetMap {
    pub fn new() -> Self {
        Self {
            // ..Default::default()
        }
    }

    // pub fn update(&mut self) {
    //     // for i in &self.eviction_self {
    //     //     self.travel.remove(i.to_owned());
    //     // }
    //     // self.eviction_self = Vec::new();
    //
    //     for i in &self.eviction {
    //         self.targets.remove(i.to_owned());
    //     }
    //     self.eviction = Vec::new();
    // }
}
