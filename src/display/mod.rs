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

mod main_app;
use crate::prelude::*;
pub use main_app::*;

use chrono::Duration;
use egui::{Color32, Context, Pos2};
use rand::Rng;
use std::f64::{consts::PI, NAN};

/// Take radian value and outup a nice 00° 00’ 00” string
pub fn pretty(a: f64) -> String {
    let degrees = a.to_degrees().trunc();
    let minutes = (a.to_degrees().fract() * 60.0).trunc().abs();
    let seconds = ((a.to_degrees().fract() * 60.0).fract() * 60.0)
        .trunc()
        .abs();
    format!("{degrees}° {minutes}’ {seconds}”")
}

/// Take a chrono::Duration and output a nice hh:mm:ss string
pub fn pretty_duration(a: Duration) -> String {
    let seconds = a.num_seconds() % 60;
    let minutes = (a.num_seconds() / 60) % 60;
    let hours = (a.num_seconds() / 60) / 60;
    format!("{:0>2}:{:0>2}:{:0>2}", hours, minutes, seconds)
}

// pub fn legend(a: f64, b: usize, range: &RangeInclusive<f64>) -> String {
//     let degrees = a.to_degrees().trunc();
//     format!("{degrees}°")
// }

// pub fn grid(range: GridInput) -> Vec<GridMark> {
//     let mut mark = Vec::new();
//     let (a, b) = range.bounds;
//     let (a, b) = (a.to_degrees().trunc() as i64, b.to_degrees().trunc() as i64);
//     let aa = (a / 10) * 10;
//     for i in (aa..b).step_by(10) {
//         mark.push(GridMark {
//             value: (i as f64).to_radians(),
//             step_size: 25.0,
//         })
//     }
//     mark
// }

/// Currently Ui on ship show heading with 5° marker + 1/3 of 5°, we can convert 5° to 300’ so it is easier (hopefully) to apply heading
pub fn borked_cig_heading(a: f64) -> String {
    let a = (a + PI + PI) % (PI + PI);
    let degrees = a.to_degrees();
    let graduation = (degrees / 5.0) as i64 * 5;
    let minutes = (degrees % 5.0 * 60.0).trunc();

    format!("{graduation}°{minutes}’")
}

/// Get a random ecolor::Color32
pub fn random_color32() -> Color32 {
    Color32::from_rgb(
        rand::thread_rng().gen(),
        rand::thread_rng().gen(),
        rand::thread_rng().gen(),
    )
}

impl Target {
    pub fn display(&mut self, ctx: &Context) {
        egui::Window::new(format!(
            "{} - {}",
            self.current_point.container_name, self.current_point.name
        ))
        .default_pos(Pos2::new(400.0, 800.0))
        .open(&mut self.widget_open)
        .show(ctx, |ui| {
            egui::Grid::new("MainGrid").show(ui, |ui| {
                ui.label("Latitute:");
                ui.label(pretty(self.current_point.latitude));
                ui.end_row();
                ui.label("Longitude:");
                ui.label(pretty(self.current_point.longitude));
                ui.end_row();
                ui.label("Altitude:");
                ui.label(format!("{:.3}km", self.current_point.altitude));
                ui.end_row();
                ui.label("Distance:");
                ui.label(format!("{:.3}km", self.current_distance));
                ui.end_row();
                ui.label("Heading:");
                ui.label(pretty(self.current_heading));
                ui.end_row();
                ui.label("CIG Heading:");
                ui.label(borked_cig_heading(self.current_heading));
                ui.end_row();
            });
        });
    }
}

impl Path {
    pub fn display(&mut self, ctx: &Context) {
        self.current_index = self.current_index.clamp(0, self.history.len());
        let current_point = if self.current_index < 1 {
            None
        } else {
            self.history.get(self.current_index - 1)
        };

        egui::Window::new(format!("Path - {}", self.name))
            .default_pos(Pos2::new(400.0, 800.0))
            .open(&mut self.widget_open)
            .show(ctx, |ui| {
                egui::Grid::new("MainGrid").show(ui, |ui| {
                    ui.horizontal(|ui| {
                        if ui.button("⏴").clicked() & (self.current_index > 0) {
                            self.current_index -= 1;
                        };
                        if ui.button("⏵").clicked() & (self.current_index < self.history.len()) {
                            self.current_index += 1;
                        };

                        ui.heading(format!("{}/{}", self.current_index, self.history.len()));
                    });
                    ui.heading(
                        current_point
                            .map(|p| p.name.to_string())
                            .unwrap_or_default(),
                    );
                    ui.end_row();
                    ui.label("Latitute:");
                    ui.label(pretty(current_point.map(|p| p.latitude).unwrap_or(NAN)));
                    ui.end_row();
                    ui.label("Longitude:");
                    ui.label(pretty(current_point.map(|p| p.longitude).unwrap_or(NAN)));
                    ui.end_row();
                    ui.label("Altitude:");
                    ui.label(format!(
                        "{:.3}km",
                        current_point.map(|p| p.altitude).unwrap_or(NAN)
                    ));
                    ui.end_row();
                    ui.label("Distance:");
                    ui.label(format!("{:.3}km", self.current_distance));
                    ui.end_row();
                    ui.label("Heading:");
                    ui.label(pretty(self.current_heading));
                    ui.end_row();
                    ui.label("CIG Heading:");
                    ui.label(borked_cig_heading(self.current_heading));
                    ui.end_row();
                    ui.label("Duration:");
                    ui.label(pretty_duration(self.duration));
                    ui.end_row();
                    ui.label("Lenght:");
                    ui.label(format!("{:.3}km", self.length));
                    ui.end_row();
                });
            });
    }
}
