use std::{collections::HashMap, f64::NAN, fs};

use crate::geolib::{get_current_container, Container, Poi, SpaceTimePosition, Vec3d};

#[derive(Default)]
pub struct WidgetPosition {
    pub space_time_position: SpaceTimePosition,
    pub container: Container,

    pub absolute_coordinates: Vec3d,
    pub local_coordinates: Vec3d,

    pub latitude: f64,
    pub longitude: f64,
    pub altitude: f64,
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
        elapsed_time_in_seconds: f64,
    ) {
        self.space_time_position = *space_time_position;
        self.absolute_coordinates = space_time_position.coordinates;
        self.container = get_current_container(&self.absolute_coordinates, database);

        self.local_coordinates = self
            .absolute_coordinates
            .transform_to_local(elapsed_time_in_seconds, &self.container);

        if self.container.name != "Space" {
            self.latitude = self.local_coordinates.latitude();
            self.longitude = self.local_coordinates.longitude();
            self.altitude = self.local_coordinates.height(&self.container);
        }
    }
}

#[derive(Default)]
pub struct WidgetTargetSelection {
    pub target_container: Container,
    pub target_poi: Poi,
    pub targets: Vec<WidgetTarget>,
}

impl WidgetTargetSelection {
    pub fn new(targets: Vec<WidgetTarget>) -> Self {
        Self {
            targets,
            ..Default::default()
        }
    }
}

#[derive(Default)]
pub struct WidgetPoi {
    pub database: HashMap<String, Container>,
    pub name: String,
}

impl WidgetPoi {
    pub fn save_current_position(&mut self, position: &WidgetPosition) {
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

        let new_poi = if (position.container.name == "Space") | (position.container.name.is_empty())
        {
            Poi {
                name: self.name.clone(),
                container: "Space".to_string(),
                coordinates: position.absolute_coordinates.to_owned(),
                quaternions: None,
                marker: None,
            }
        } else {
            Poi {
                name: self.name.clone(),
                container: position.container.name.clone(),
                coordinates: position.local_coordinates.to_owned(),
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

#[derive(Default)]
pub struct WidgetTarget {
    pub open: bool,
    pub target: Poi,
}
