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

use egui::{
    color_picker::color_picker_color32, CollapsingHeader, Color32, ComboBox, Context, Grid,
    RichText, TextEdit, TopBottomPanel, Ui,
};
use egui_plot::{Line, Plot, Points};

/// Dirty (but working way) too get snapped point on graph see [issue](https://github.com/emilk/egui/discussions/1778)
static mut SMARTY: String = String::new();

impl MyEguiApp {
    pub fn display(&mut self, ctx: &Context) {
        // Display floating widget
        for (_, path) in self.global_paths.iter_mut() {
            path.display(ctx)
        }

        for target in self.global_targets.iter_mut() {
            target.display(ctx)
        }

        self.display_global_store(ctx);

        // Display top row
        self.display_top(ctx);

        // Display side column
        self.display_side(ctx);

        // Display main map
        self.display_map(ctx);
    }

    fn display_global_store(&mut self, ctx: &Context) {
        egui::Window::new("Last Points")
            .open(&mut self.global_history_widget)
            .show(ctx, |ui| {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    // ui.heading("lolu");
                    let mut eviction = None;
                    for (index, point) in self.global_history.iter().enumerate().rev() {
                        ui.horizontal(|ui| {
                            ui.spacing_mut().item_spacing = egui::vec2(1.0, 1.0);
                            if ui.button("❌").clicked() {
                                eviction = Some(index);
                            };
                            if ui.button("⌖").clicked() {
                                self.global_history_index = index;
                            };
                            if ui.button("Add").clicked() {
                                if let Some(path) = self.global_paths.get_mut(&self.path_selector) {
                                    if path.history.is_empty() {
                                        path.history.push(point.clone());
                                    } else {
                                        path.history.insert(path.current_index, point.clone());
                                    }
                                    path.current_index += 1;
                                }
                            };
                            ui.label(&point.name);
                        });
                    }
                    if let Some(i) = eviction {
                        self.global_history.remove(i);
                    }
                });
            });
    }

    fn display_side(&mut self, ctx: &Context) {
        egui::SidePanel::left("my_left_panel").show(ctx, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                // Display Targets
                CollapsingHeader::new(RichText::new("Targets").heading())
                    .default_open(true)
                    .show(ui, |ui| {
                        let mut eviction = None;
                        let mut focused = None;
                        for (index, target) in self.global_targets.iter_mut().enumerate() {
                            ui.horizontal(|ui| {
                                ui.spacing_mut().item_spacing = egui::vec2(1.0, 1.0);
                                if ui.button("❌").clicked() {
                                    eviction = Some(index);
                                };
                                if ui.button("👁").clicked() {
                                    target.widget_open = !target.widget_open;
                                };
                                if ui.button("⌖").clicked() {
                                    focused = Some(target.current_point.clone());
                                };
                                ui.label(&target.current_point.name);
                            });
                        }
                        if let Some(i) = eviction {
                            self.global_targets.remove(i);
                        }
                        if let Some(point) = focused {
                            self.add_to_global(&point);
                        }
                    });

                // Display Paths
                ui.heading("Paths");
                let mut confirm_eviction = Vec::new();
                let mut focused = None;

                for path in self.global_paths.values_mut() {
                    let mut eviction = false;
                    let mut eviction_point = None;
                    let mut up = None;
                    let mut down = None;
                    let len = path.history.len();
                    let id = ui.make_persistent_id(path.name.to_string());

                    egui::collapsing_header::CollapsingState::load_with_default_open(
                        ui.ctx(),
                        id,
                        false,
                    )
                    .show_header(ui, |ui| {
                        ui.spacing_mut().item_spacing = egui::vec2(1.0, 1.0);

                        ui.heading(&path.name);

                        if ui.button("❌").clicked() {
                            eviction = true;
                        };
                        if ui.button("🗺").clicked() {
                            path.widget_open = true;
                        };
                        if ui.button("👁").clicked() {
                            path.map_displayed = !path.map_displayed;
                        };

                        ui.color_edit_button_srgba(&mut path.map_color);

                        ui.add(
                            egui::DragValue::new(&mut path.map_radius)
                                .speed(0.1)
                                .clamp_range(0..=10),
                        );
                    })
                    .body(|ui| {
                        for (i, p) in path.history.iter_mut().enumerate() {
                            ui.spacing_mut().item_spacing = egui::vec2(1.0, 1.0);
                            ui.horizontal(|ui| {
                                ui.menu_button("⚙", |ui| {
                                    ui.horizontal(|ui| {
                                        ui.spacing_mut().item_spacing = egui::vec2(1.0, 1.0);

                                        if ui.button("⌖").clicked() {
                                            path.widget_open = true;
                                            path.current_index = i + 1;
                                            focused = Some(p.clone());
                                            ui.close_menu();
                                        };
                                    });
                                    let mut color = p.color.unwrap_or(path.map_color);

                                    let color_changed = color_picker_color32(
                                        ui,
                                        &mut color,
                                        egui::color_picker::Alpha::Opaque,
                                    );
                                    if color_changed {
                                        p.color = Some(color);
                                    }
                                });
                                if ui.button("❌").clicked() {
                                    eviction_point = Some(i);
                                };
                                if ui.button("⏶").clicked() & (len > 1) {
                                    up = Some(i);
                                };
                                if ui.button("⏷").clicked() & (len > 1) {
                                    down = Some(i);
                                };
                                if path.current_index == i + 1 {
                                    ui.label(RichText::new(&p.name).strong());
                                } else {
                                    ui.label(&p.name);
                                }
                            });
                        }
                    });

                    if let Some(i) = eviction_point {
                        path.history.remove(i);
                    } else if let Some(i) = up {
                        let point = path.history.remove(i);
                        path.history.insert(i.max(1) - 1, point)
                    } else if let Some(i) = down {
                        let point = path.history.remove(i);
                        path.history.insert(i.min(len - 2) + 1, point)
                    }

                    if eviction {
                        // Clic on x for path
                        if path.history.is_empty() & (&path.name != "Self") {
                            // Is already empty remove path from global_paths
                            confirm_eviction.push(path.name.to_string());
                        } else {
                            // Empty all point
                            path.history = Vec::new();
                        }
                    }
                }

                for path_name in confirm_eviction {
                    self.global_paths.remove(&path_name);
                }
                if let Some(point) = focused {
                    self.add_to_global(&point);
                }
            });
        });
    }

    fn display_top(&mut self, ctx: &Context) {
        TopBottomPanel::top("top_panel").show(ctx, |ui| {
            ui.horizontal(|ui| {
                self.display_top_left(ui);
                ui.separator();

                self.display_top_middle(ui);
                ui.separator();

                self.display_top_right(ui);
            });
        });
    }

    fn display_top_left(&mut self, ui: &mut Ui) {
        ui.vertical(|ui| {
            ui.horizontal(|ui| {
                ui.heading("Self Position");
                ui.spinner();
            });

            if let Some(current_path) = self.global_paths.get(&self.path_selector) {
                if current_path.current_index == 0 {
                    ui.heading("No Position highlighted !");
                } else if let Some(position) =
                    current_path.history.get(current_path.current_index - 1)
                {
                    egui::Grid::new("SelfPosition").show(ui, |ui| {
                        ui.label("Timestamp:");
                        ui.label(format!("{}", position.space_time_position.timestamp));
                        ui.end_row();
                        ui.label("Coordinates_X:");
                        ui.label(format!("{}", position.space_time_position.coordinates.x));
                        ui.end_row();
                        ui.label("Coordinates_Y:");
                        ui.label(format!("{}", position.space_time_position.coordinates.y));
                        ui.end_row();
                        ui.label("Coordinates_Z:");
                        ui.label(format!("{}", position.space_time_position.coordinates.z));
                        ui.end_row();
                        ui.label("Container:");
                        ui.label(&position.container_name);
                        ui.end_row();
                        ui.label("Latitute:");
                        ui.label(pretty(position.latitude));
                        ui.end_row();
                        ui.label("Longitude:");
                        ui.label(pretty(position.longitude));
                        ui.end_row();
                        ui.label("Altitude:");
                        ui.label(format!("{:.3}km", position.altitude));
                        ui.end_row();
                    });
                }
            } else {
                ui.heading("No Position 😕");
            }
            egui::Grid::new("InfferedHeading").show(ui, |ui| {
                ui.label("Current_heading:");
                ui.label(borked_cig_heading(self.current_heading));
                ui.end_row();
            });
        });
    }

    fn display_top_middle(&mut self, ui: &mut Ui) {
        ui.vertical(|ui| {
            let mut eviction = false;

            ui.horizontal(|ui| {
                ui.heading("Path History");
                ComboBox::from_id_source("Path")
                    .selected_text(&self.path_selector)
                    .show_ui(ui, |ui| {
                        for e in self.global_paths.keys() {
                            ui.selectable_value(&mut self.path_selector, e.clone(), e);
                        }
                    });
            });

            if let Some(current_path) = self.global_paths.get_mut(&self.path_selector) {
                let len = current_path.history.len();
                if current_path.current_index == 0 {
                    ui.horizontal(|ui| {
                        ui.spacing_mut().item_spacing = egui::vec2(1.0, 1.0);
                        if ui.button("⏴").clicked() & (current_path.current_index > 0) {
                            current_path.current_index -= 1;
                        };
                        if ui.button("⏵").clicked() & (current_path.current_index < len) {
                            current_path.current_index += 1;
                        };

                        ui.heading(format!("😕 {}/{}", current_path.current_index, len,));
                    });
                } else if let Some(position) =
                    current_path.history.get_mut(current_path.current_index - 1)
                {
                    ui.horizontal(|ui| {
                        ui.spacing_mut().item_spacing = egui::vec2(1.0, 1.0);
                        if ui.button("❌").clicked() {
                            eviction = true;
                        };
                        if ui.button("⏴").clicked() & (current_path.current_index > 0) {
                            current_path.current_index -= 1;
                        };
                        if ui.button("⏵").clicked() & (current_path.current_index < len) {
                            current_path.current_index += 1;
                        };

                        ui.heading(format!("{}/{}:", current_path.current_index, len,));

                        ui.heading(position.name.to_string());
                    });
                    ui.horizontal(|ui| {
                        ui.add(TextEdit::singleline(&mut position.name).hint_text("No_name"));

                        if ui.button("Save as POI").clicked() {
                            let new_poi = save_to_poi(position);
                            // Add to database
                            self.database
                                .get_mut(&new_poi.container)
                                .unwrap()
                                .poi
                                .insert(new_poi.name.clone(), new_poi);
                        };
                        ui.end_row();
                    });
                }
            } else {
                ui.heading("No Position 😕");
            };
            if eviction {
                if let Some(current_path) = self.global_paths.get_mut(&self.path_selector) {
                    current_path.history.remove(current_path.current_index - 1);
                }
            }

            // ui.separator(); // BUG fill entire right panel
            ui.label("---------------------------");
            ui.vertical(|ui| {
                ui.heading("Path I/O");

                if let Some(path) = self.global_paths.get(&self.path_selector) {
                    if ui.button("Export Path").clicked() {
                        save_history(&self.path_name_io, &path.history);
                    }
                } else {
                    ui.add_enabled(false, egui::Button::new("Export Path"));
                }

                if ui.button("Import Path").clicked() {
                    let mut path = Path::new(self.path_name_io.to_string());
                    path.map_color = random_color32();
                    path.history = import_history(&self.path_name_io);
                    self.global_paths
                        .insert(self.path_name_io.to_string(), path);
                };
                ui.add(TextEdit::singleline(&mut self.path_name_io).hint_text("Path Name"));
            });
        });
    }

    fn display_top_right(&mut self, ui: &mut Ui) {
        ui.vertical(|ui| {
            ui.heading("Target Selector");
            Grid::new("TargetSelector").show(ui, |ui| {
                ui.label("Container");
                ComboBox::from_id_source("Container")
                    .selected_text(&self.target_selector_container)
                    .show_ui(ui, |ui| {
                        for container in self.database.values() {
                            ui.selectable_value(
                                &mut self.target_selector_container,
                                container.name.clone(),
                                container.name.clone(),
                            );
                        }
                    });
                ui.end_row();

                ui.label("Poi");
                ComboBox::from_id_source("Poi")
                    .selected_text(&self.target_selector_poi)
                    .show_ui(ui, |ui| {
                        if self.database.contains_key(&self.target_selector_container) {
                            for poi in self
                                .database
                                .get(&self.target_selector_container)
                                .unwrap()
                                .poi
                                .values()
                            {
                                ui.selectable_value(
                                    &mut self.target_selector_poi,
                                    poi.name.clone(),
                                    poi.name.clone(),
                                );
                            }
                        }
                    });
                ui.end_row();

                if ui.button("Add Target").clicked()
                    & self.database.contains_key(&self.target_selector_container)
                {
                    if let Some(poi) = self
                        .database
                        .get(&self.target_selector_container)
                        .unwrap()
                        .poi
                        .get(&self.target_selector_poi)
                    {
                        self.global_targets.push(Target::new(poi, &self.database));
                        // TODO check for duplicate !
                    }
                };

                ui.end_row();
            });
            ui.label("------------------------");
            ui.checkbox(&mut self.path_add_point, "Auto add point");

            if ui.button("GlobalStore").clicked() {
                self.global_history_widget = !self.global_history_widget;
            }
        });
    }

    fn display_map(&mut self, ctx: &Context) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Map");
            // TODO get map from scdatatools
            let plot_response = Plot::new("my_plot")
                .data_aspect(1.0)
                .include_x(-180)
                .include_x(180)
                .include_y(90)
                .include_y(-90)
                .label_formatter(|name, value| {
                    unsafe {
                        SMARTY = name.to_string();
                    }
                    if !name.is_empty() {
                        format!(
                            "{name}\n{}\n{}",
                            pretty(value.y.to_radians()),
                            pretty(value.x.to_radians())
                        )
                    } else {
                        "".to_owned()
                    }
                })
                .show(ui, |plot_ui| {
                    // Draw Targets in map
                    for target in &self.global_targets {
                        let c = [
                            target.current_point.longitude.to_degrees(),
                            target.current_point.latitude.to_degrees(),
                        ];
                        plot_ui.points(
                            Points::new(c)
                                .name(&target.current_point.name)
                                .radius(target.map_radius)
                                .shape(target.map_shape)
                                .color(target.map_color),
                        );
                    }

                    // Draw Paths in map
                    for path in self.global_paths.values() {
                        if path.map_displayed {
                            // Accumulator
                            let mut path_line = Vec::new();
                            let mut path_point = Vec::new();
                            for (index, point) in path.history.iter().enumerate() {
                                let c = [
                                    point.local_coordinates.longitude().to_degrees(),
                                    point.local_coordinates.latitude().to_degrees(),
                                ];
                                path_line.push(c);
                                path_point.push(if path.current_index == index + 1 {
                                    let highlight_color = Color32::from_rgb(
                                        255 - path.map_color.r(),
                                        255 - path.map_color.g(),
                                        255 - path.map_color.b(),
                                    );
                                    Points::new(c)
                                        .name(&point.name)
                                        .radius(path.map_radius)
                                        .color(highlight_color)
                                        .shape(path.map_shape)
                                        .highlight(true)
                                } else {
                                    Points::new(c)
                                        .name(&point.name)
                                        .radius(path.map_radius)
                                        .color(point.color.unwrap_or(path.map_color))
                                        .shape(path.map_shape)
                                });
                            }
                            for point in path_point {
                                plot_ui.points(point);
                            }
                            plot_ui.line(
                                Line::new(path_line)
                                    .name(&path.name) //BUG highlight point get pathname on display, side effect frow drawing path...
                                    .width(1.5)
                                    .color(path.map_color),
                            );
                        }
                    }
                });

            // Handle interaction
            let snapped_point: String;
            unsafe {
                snapped_point = SMARTY.clone();
                SMARTY = String::new();
            }
            if plot_response
                .response
                .clicked_by(egui::PointerButton::Primary)
            {
                let mut focused = None;
                for path in self.global_paths.values_mut() {
                    for (i, point) in path.history.iter().enumerate() {
                        if point.name == snapped_point {
                            path.current_index = i + 1;
                            focused = Some(point.clone());
                        }
                    }
                }
                if let Some(point) = focused {
                    self.add_to_global(&point);
                }
            }
            if plot_response
                .response
                .clicked_by(egui::PointerButton::Middle)
            {
                let new_point = plot_response
                    .transform
                    .value_from_position(ctx.pointer_interact_pos().unwrap_or_default());

                let latitude = new_point.y.to_radians();
                let longitude = new_point.x.to_radians();
                self.new_coordinates_from_map(latitude, longitude);
            }
        });
    }
}
