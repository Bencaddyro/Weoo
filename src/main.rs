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

// #![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

//! # Weoo
//!
//! A StarCitizen navigation tool !

/// main egui application definition and function
mod application;
/// All function to do math and define geometry struct
mod data_math;
/// All function to draw should be here
mod display;
/// Input & ouput functions belong here
mod input_output;

#[doc(hidden)]
mod prelude {
    pub use crate::{application::*, data_math::*, display::*, input_output::*};
}

use crate::prelude::*;
use chrono::{DateTime, TimeZone, Utc};
use once_cell::sync::Lazy;

// Somewhere on Daymar
// Coordinates: x:-18930379393.98 y:-2610297380.75 z:210614.307494
// Coordinates: x:-18930479393.98 y:-2610297380.75 z:210614.307494
// Coordinates: x:-18930579393.98 y:-2610297380.75 z:210614.307494
// Coordinates: x:-18930679393.98 y:-2610297380.75 z:210614.307494
// Coordinates: x:-18930779393.98 y:-2610297380.75 z:210614.307494

/// Used to compute elpased time since reference time
static REFERENCE_TIME: Lazy<DateTime<Utc>> =
    Lazy::new(|| Utc.with_ymd_and_hms(2020, 1, 1, 0, 0, 0).unwrap());

/// Main entrypoint, run MyEguiApp (name to be changed someday)
fn main() -> eframe::Result<()> {
    let native_options = eframe::NativeOptions::default();
    eframe::run_native(
        "Weoo Nav Tool",
        native_options,
        Box::new(|cc| Box::new(MyEguiApp::new(cc))),
    )
}
