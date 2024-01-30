// Weoo, a StarCitizen navigation tool
// Copyright (C) 2024 Benoît Fournier benoit.fournier@clever-cloud.com
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.
//

use crate::prelude::*;

use chrono::Duration;
use egui::Color32;
use egui_plot::MarkerShape;
use std::{
    collections::{BTreeMap, HashMap},
    f64::{consts::PI, NAN},
};

pub type Paths = HashMap<String, Path>;
pub type Targets = Vec<Target>;
pub type Database = BTreeMap<String, OldContainer>;

#[derive(Debug)]
pub struct Target {
    // Display on Map info
    pub map_color: Color32,
    pub map_shape: MarkerShape,
    pub map_radius: f32,

    // Display on widget info
    pub widget_open: bool,
    pub current_point: ProcessedPosition,
    pub current_distance: f64,
    pub current_heading: f64,
}

#[derive(Debug, Clone)]
pub struct Path {
    pub name: String,
    pub history: Vec<ProcessedPosition>,
    pub duration: Duration,
    pub length: f64,

    // Display on Map info
    pub map_color: Color32,
    pub map_shape: MarkerShape,
    pub map_radius: f32,
    pub map_displayed: bool,

    // Display on widget info
    pub widget_open: bool,

    // Display info for current highligted point
    pub current_index: usize, // index = 0 mean no highlight !
    pub current_distance: f64,
    pub current_heading: f64,
}

impl Target {
    pub fn new(target: &OldPoi, database: &NewDatabase) -> Self {
        Self {
            widget_open: true,
            current_point: poi_to_processed_point(target, database),
            current_distance: NAN,
            current_heading: NAN,
            map_color: random_color32(),
            map_shape: MarkerShape::Diamond,
            map_radius: 4.0,
        }
    }

    pub fn update(&mut self, database: &NewDatabase, current_position: Option<&ProcessedPosition>) {//TODO new database
        if let Some(complete_position) = current_position {
            let target_container = database.containers.get(&self.current_point.container_name).unwrap();
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
                .current_point
                .local_coordinates
                .rotate(target_rotation_state_in_degrees.to_radians());

            // #---------------------------------------------------Distance to target----------------------------------------------------------
            let delta_distance =
                if complete_position.container_name == self.current_point.container_name {
                    // println!("{:?}\n{:?}\n{:?}\n\n ",self.current_point.local_coordinates - complete_position.local_coordinates, self.current_point.local_coordinates, complete_position.local_coordinates );
                    self.current_point.local_coordinates - complete_position.local_coordinates
                } else {
                    // println!("not same container ! houlala");
                    // println!("{} - {} ",complete_position.container_name, self.current_point.container_name );
                    target_rotated_coordinates + target_container.coordinates
                    // - complete_position.local_coordinates // why this ?
                    // + complete_position.absolute_coordinates // and why a + ?
                    - complete_position.space_time_position.coordinates
                };
            self.current_distance = delta_distance.norm();

            // #----------------------------------------------------------Heading--------------------------------------------------------------
            // If planetary !
            self.current_heading = (complete_position
                // .space_time_position
                // .coordinates
                .local_coordinates
                .loxodromie_to(self.current_point.local_coordinates)
                + 2.0 * PI)
                % (2.0 * PI);
        }
    }
}

impl Path {
    pub fn new(name: String) -> Path {
        Path {
            name,
            history: Vec::new(),
            duration: Duration::zero(),
            length: 0.0,

            map_color: Color32::DARK_GRAY,
            map_shape: MarkerShape::Circle,
            map_radius: 3.0,
            map_displayed: true,

            widget_open: false,
            current_index: 0,
            current_distance: 0.0,
            current_heading: 0.0,
        }
    }

    pub fn update(&mut self, database: &NewDatabase, complete_position: Option<&ProcessedPosition>) {
        // Update path lenght
        self.length = 0.0;
        if !self.history.is_empty() {
            for i in 1..self.history.len() {
                self.length += (self.history[i - 1].local_coordinates
                    - self.history[i].local_coordinates)
                    .norm();
            }
        }
        // Update path duration
        if let Some(point) = self.history.last() {
            self.duration =
                point.space_time_position.timestamp - self.history[0].space_time_position.timestamp;
        };

        // Update hightligt to point
        if let Some(complete_position) = complete_position {
            if !self.history.is_empty() {
                let index = self.current_index.clamp(1, self.history.len()) - 1;
                let target_local_coordinates = self.history[index].local_coordinates;

                let target_container = database.containers.get(&self.history[0].container_name).unwrap();
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
                let delta_distance =
                    if complete_position.container_name == self.history[index].container_name {
                        target_local_coordinates - complete_position.local_coordinates
                    } else {
                        target_rotated_coordinates + target_container.coordinates
                    // - complete_position.local_coordinates // why this ?
                    // + complete_position.absolute_coordinates // and why a + ?
                    - complete_position.space_time_position.coordinates
                    };
                self.current_distance = delta_distance.norm();

                // #----------------------------------------------------------Heading--------------------------------------------------------------
                // If planetary !
                self.current_heading = (complete_position
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
