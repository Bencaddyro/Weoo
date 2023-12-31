use chrono::Duration;

use crate::geolib::{Container, Path, Poi, ProcessedPosition, Vec3d};
use std::{
    collections::{BTreeMap, HashMap},
    f64::consts::PI,
};

#[derive(Clone, Default)]
pub struct WidgetTopPosition {
    // History
    pub history_name: String,

    // Target selector
    pub target_container: Container,
    pub target_poi: Poi,

    // Add showlocation to path ?
    pub auto_add_point: bool,
}

#[derive(Default)]
pub struct WidgetTargets {
    pub targets: Vec<WidgetTarget>,
}

#[derive(Default, Debug)]
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

#[derive(Debug)]
pub struct WidgetPath {
    pub open: bool,
    pub index: usize,
    pub history: String, //path name
    pub latitude: f64,
    pub longitude: f64,
    pub altitude: f64,
    pub distance: f64,
    pub heading: f64,
    pub duration: Duration,
    pub length: f64,
}

#[derive(Default)]
pub struct WidgetMap {
    // Nothing for now
    // Futur: Map parameters : Container selector, 2D / 3D, height map, biome
}

impl WidgetTargets {
    pub fn new(targets: Vec<WidgetTarget>) -> Self {
        Self { targets }
    }
}

impl WidgetPath {
    pub fn update(
        &mut self,
        database: &BTreeMap<String, Container>,
        complete_position: Option<&ProcessedPosition>,
        paths: &HashMap<String, Path>,
    ) {
        if let Some(path) = paths.get(&self.history) {
            // Update path lenght
            self.length = 0.0;
            for i in 1..path.history.len() {
                self.length += (path.history[i - 1].local_coordinates
                    - path.history[i].local_coordinates)
                    .norm();
            }
            // Update path duration
            self.duration = path.history.last().unwrap().space_time_position.timestamp
                - path.history[0].space_time_position.timestamp;

            if let Some(complete_position) = complete_position {
                let target_local_coordinates = path.history[self.index].local_coordinates;

                let target_container = database.get(&path.history[0].container_name).unwrap();
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
                let target_rotated_coordinates =
                    target_local_coordinates.rotate(target_rotation_state_in_degrees.to_radians());

                // #---------------------------------------------------Distance to target----------------------------------------------------------
                let delta_distance = if complete_position.container_name
                    == path.history[self.index].container_name
                {
                    target_local_coordinates - complete_position.local_coordinates
                } else {
                    target_rotated_coordinates + target_container.coordinates
                    // - complete_position.local_coordinates // why this ?
                    // + complete_position.absolute_coordinates // and why a + ?
                    - complete_position.space_time_position.coordinates
                };
                self.distance = delta_distance.norm();

                // #----------------------------------------------------------Heading--------------------------------------------------------------
                // If planetary !
                self.heading = (complete_position
                    // .space_time_position
                    // .coordinates
                    .local_coordinates
                    .loxodromie_to(target_local_coordinates)
                    + 2.0 * PI)
                    % (2.0 * PI);
            }
        }
    }
}

impl WidgetTarget {
    pub fn new(target: Poi, database: &BTreeMap<String, Container>) -> Self {
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
        database: &BTreeMap<String, Container>,
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
            self.delta_distance = if complete_position.container_name == self.target.container {
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
                // .space_time_position
                // .coordinates
                .local_coordinates
                .loxodromie_to(self.target.coordinates)
                + 2.0 * PI)
                % (2.0 * PI);
        }
    }
}
