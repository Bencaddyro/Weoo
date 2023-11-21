use crate::geolib::{get_current_container, Container, Poi, SpaceTimePosition, Vec3d};
use std::{collections::HashMap, f64::NAN, fs};

#[derive(Clone, Default)]
pub struct WidgetPosition {
    pub space_time_position: SpaceTimePosition,
    pub container: Container,

    pub absolute_coordinates: Vec3d,
    pub local_coordinates: Vec3d,

    pub latitude: f64,
    pub longitude: f64,
    pub altitude: f64,
}

#[derive(Default)]
pub struct WidgetPoi {
    pub database: HashMap<String, Container>,
    pub name: String,
    pub position: WidgetPosition,
}

#[derive(Default)]
pub struct WidgetTargetSelection {
    pub target_container: Container,
    pub target_poi: Poi,
    pub targets: HashMap<String, WidgetTarget>,
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
    pub open: bool,
    pub targets: HashMap<String, Poi>,
}

impl WidgetPosition {
    pub fn new() -> Self {
        Self {
            latitude: NAN,
            longitude: NAN,
            altitude: NAN,
            ..Default::default()
        }
    }

    pub fn update(
        &mut self,
        space_time_position: &SpaceTimePosition,
        database: &HashMap<String, Container>,
        time_elapsed: f64,
    ) {
        self.space_time_position = *space_time_position;
        self.absolute_coordinates = space_time_position.coordinates;
        self.container = get_current_container(&self.absolute_coordinates, database);

        self.local_coordinates = self
            .absolute_coordinates
            .transform_to_local(time_elapsed, &self.container);

        if self.container.name != "Space" {
            self.latitude = self.local_coordinates.latitude();
            self.longitude = self.local_coordinates.longitude();
            self.altitude = self.local_coordinates.altitude(&self.container);
        }
    }
}

impl WidgetPoi {
    pub fn update(&mut self, database: &HashMap<String, Container>, position: &WidgetPosition) {
        self.position = position.clone();
        self.database = database.clone();
    }

    pub fn save_current_position(&mut self) {
        let mut custom_pois: HashMap<String, Poi>;
        // Open Custom Poi file
        if let Ok(file) = fs::File::open("CustomPoi.json") {
            custom_pois = serde_json::from_reader(file).expect("file should be proper JSON");
        } else {
            println!("No file");
            custom_pois = HashMap::new();
        };

        // Search for existing Poi with this name
        if custom_pois.contains_key(&self.name) {
            println!("Poi already exist, default override")
        }

        let new_poi = if (self.position.container.name == "Space")
            | (self.position.container.name.is_empty())
        {
            Poi {
                name: self.name.clone(),
                container: "Space".to_string(),
                coordinates: self.position.absolute_coordinates.to_owned(),
                quaternions: None,
                marker: None,
            }
        } else {
            Poi {
                name: self.name.clone(),
                container: self.position.container.name.clone(),
                coordinates: self.position.local_coordinates.to_owned(),
                quaternions: None,
                marker: None,
            }
        };
        // Add to set
        custom_pois.insert(self.name.clone(), new_poi.clone());

        // Add to database
        self.database
            .get_mut(&new_poi.container)
            .unwrap()
            .poi
            .insert(self.name.clone(), new_poi);

        // Write files
        let mut file = std::fs::File::create("CustomPoi.json").expect("This should work");
        serde_json::to_writer_pretty(&mut file, &custom_pois)
            .expect("Fail to write cutom poi json");
    }
}

impl WidgetTargetSelection {
    pub fn new(targets: HashMap<String, WidgetTarget>) -> Self {
        Self {
            targets,
            ..Default::default()
        }
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
                    - complete_position.absolute_coordinates
        };
        self.distance = self.delta_distance.norm();

        // #----------------------------------------------------------Heading--------------------------------------------------------------
        // If planetary !
        self.heading = (complete_position
            .local_coordinates
            .loxodromie_to(self.target.coordinates)
            .to_degrees()
            + 360.0)
            % 360.0;
    }
}

impl WidgetMap {
    pub fn remove_targets(&mut self, target: &Poi) {
        self.targets.remove(&target.name);
    }
}
