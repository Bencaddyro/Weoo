use crate::{
    geolib::{Container, Path},
    iolib::{import_history, save_history, save_to_poi},
    mainlib::{WidgetMap, WidgetTarget, WidgetTargets, WidgetTopPosition, WidgetPath},
};
use egui::{Color32, ComboBox, Context, Grid, Pos2, TextEdit, TopBottomPanel};
use egui_plot::{Line, MarkerShape, Plot, Points};
use std::{
    collections::{BTreeMap, HashMap},
    f64::consts::PI,
};

pub fn pretty(a: f64) -> String {
    let degrees = a.to_degrees().trunc();
    let minutes = (a.to_degrees().fract() * 60.0).trunc().abs();
    let seconds = ((a.to_degrees().fract() * 60.0).fract() * 60.0)
        .trunc()
        .abs();
    format!("{degrees}° {minutes}’ {seconds}”")
}

pub fn borked_cig_heading(a: f64) -> String {
    let degrees = a.to_degrees();
    let graduation = (degrees / 5.0) as i64 * 5;
    let minutes = (degrees % 5.0 * 60.0).trunc();

    format!("{graduation}°{minutes}’")
}
use rand::Rng;
// ui.label("Debug:"); TODO Debug feature
// ui.add(egui::TextEdit::multiline(&mut format!("Timestamp: {}\nCoordinates: x:{} y:{} z:{}",
//                                     position.space_time_position.timestamp,
//                                     position.space_time_position.coordinates.x,
//                                     position.space_time_position.coordinates.y,
//                                     position.space_time_position.coordinates.z,
//                                             )));
// ui.end_row();

impl WidgetTopPosition {
    pub fn display(
        &mut self,
        ctx: &Context,
        database: &mut BTreeMap<String, Container>,
        index: &mut usize,
        displayed_path: &mut String,
        targets: &mut WidgetTargets,
        paths: &mut HashMap<String, Path>,
    ) {
        let len = paths.get_mut(displayed_path).unwrap().history.len();

        TopBottomPanel::top("top_panel").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.vertical(|ui| {
                    // Current position
                    ui.horizontal(|ui| {
                        ui.heading("Self Position");
                        ui.spinner();
                    });

                    if let Some(position) = paths
                        .get_mut(displayed_path)
                        .unwrap()
                        .history
                        .get_mut(*index)
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
                            ui.label(position.container_name.clone());
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
                    } else {
                        ui.heading("No Position 😕");
                    }
                });

                ui.separator();
                // History

                ui.vertical(|ui| {
                    let mut eviction = false;

                    ui.horizontal(|ui| {
                        ui.heading("Path History");
                        ComboBox::from_id_source("Path")
                            .selected_text(displayed_path.to_string())
                            .show_ui(ui, |ui| {
                                for e in paths.keys() {
                                    ui.selectable_value(displayed_path, e.clone(), e);
                                }
                            });
                    });

                    if let Some(position) = paths
                        .get_mut(displayed_path)
                        .unwrap()
                        .history
                        .get_mut(*index)
                    {
                        ui.horizontal(|ui| {
                            if ui.button("❌").clicked() {
                                eviction = true;
                            };
                            if ui.button("⏴").clicked() & (*index > 0) {
                                *index -= 1;
                            };
                            if ui.button("⏵").clicked() & (*index + 1 < len) {
                                *index += 1;
                            };

                            ui.heading(format!("{}/{}:", *index + 1, len,));

                            ui.heading(position.name.to_string());
                        });
                        ui.horizontal(|ui| {
                            ui.add(TextEdit::singleline(&mut position.name).hint_text("No_name"));

                            if ui.button("Save as POI").clicked() {
                                let new_poi = save_to_poi(position);
                                // Add to database
                                database
                                    .get_mut(&new_poi.container)
                                    .unwrap()
                                    .poi
                                    .insert(new_poi.name.clone(), new_poi);
                            };

                            ui.end_row();
                        });
                    } else {
                        ui.heading("No Position 😕");
                    };
                    if eviction {
                        paths
                            .get_mut(displayed_path)
                            .unwrap()
                            .history
                            .remove(*index);
                    }

                    // ui.separator(); // BUG fill entire right panel
                    ui.label("---------------------------");
                    ui.vertical(|ui| {
                        ui.heading("Path I/O");

                        if ui.button("Export Path").clicked() {
                            save_history(
                                &self.history_name,
                                &paths.get_mut(displayed_path).unwrap().history,
                            );
                        };
                        if ui.button("Import Path").clicked() {
                            paths.insert(
                                self.history_name.to_owned(),
                                Path {
                                    name: self.history_name.to_owned(),
                                    color: Color32::from_rgb(
                                        rand::thread_rng().gen(),
                                        rand::thread_rng().gen(),
                                        rand::thread_rng().gen(),
                                    ),
                                    history: import_history(&self.history_name),
                                    shape: MarkerShape::Diamond,
                                },
                            );
                        };
                        ui.add(TextEdit::singleline(&mut self.history_name).hint_text("Path Name"));
                    });
                });

                ui.separator();
                // Target selection

                ui.vertical(|ui| {
                    ui.heading("Target Selector");
                    Grid::new("TargetSelector").show(ui, |ui| {
                        ui.label("Container");
                        ComboBox::from_id_source("Container")
                            .selected_text(self.target_container.name.clone())
                            .show_ui(ui, |ui| {
                                for container in database.values() {
                                    ui.selectable_value(
                                        &mut self.target_container,
                                        container.clone(),
                                        container.name.clone(),
                                    );
                                }
                            });
                        ui.end_row();

                        ui.label("Poi");
                        ComboBox::from_id_source("Poi")
                            .selected_text(self.target_poi.name.clone())
                            .show_ui(ui, |ui| {
                                if database.contains_key(&self.target_container.name) {
                                    for poi in database
                                        .get(&self.target_container.name)
                                        .unwrap()
                                        .poi
                                        .values()
                                    {
                                        ui.selectable_value(
                                            &mut self.target_poi,
                                            poi.clone(),
                                            poi.name.clone(),
                                        );
                                    }
                                }
                            });
                        ui.end_row();

                        if ui.button("Add Target").clicked()
                            & database.contains_key(&self.target_poi.container)
                        {
                            targets.targets.push(
                                // TODO avoid duplicate
                                WidgetTarget::new(self.target_poi.clone(), database),
                            );
                        };

                        ui.end_row();
                    });
                });
            });
        });
    }
}

impl WidgetTargets {
    pub fn display(
        &mut self,
        ctx: &egui::Context,
        index: &mut usize,
        displayed_path: &mut String,
        paths: &mut HashMap<String, Path>,
        targets_path: &mut HashMap<String, WidgetPath>,
    ) {
        egui::SidePanel::left("my_left_panel").show(ctx, |ui| {
            ui.heading("Targets");

            let mut eviction = Vec::new();
            for (i, e) in self.targets.iter_mut().enumerate() {
                ui.horizontal(|ui| {
                    if ui.button("❌").clicked() {
                        eviction.push(i);
                    };
                    if ui.button("👁").clicked() {
                        e.open = !e.open;
                    };
                    // if ui.button("⏶").clicked() { };
                    // if ui.button("⏷").clicked() { };

                    ui.label(&e.target.name);
                });

                e.display(ctx);
            }
            for i in eviction {
                self.targets.remove(i);
            }

            ui.heading("Self Positions");

            let mut eviction = Vec::new();


            for (i, p) in paths.get_mut("Self").unwrap().history.iter().enumerate() {
                ui.horizontal(|ui| {
                    if ui.button("❌").clicked() {
                        eviction.push(i)
                    };
                    // if ui.button("⏶").clicked() { };
                    // if ui.button("⏷").clicked() { };

                    ui.label(p.name.clone());
                });
            }

            for i in eviction {
                paths.get_mut("Self").unwrap().history.remove(i);
            }

            // clamp index if deletion
            let len = if paths.get_mut(displayed_path).unwrap().history.is_empty() {
                0
            } else {
                paths.get_mut(displayed_path).unwrap().history.len() - 1
            };
            *index = (*index).min(len);

            ui.heading("Paths");

            let mut eviction_path = None;

            for (k, path) in paths.iter_mut() {
                if k != "Self" {
                    ui.horizontal(|ui| {
                        if ui.button("❌").clicked() {
                            eviction_path = Some(k.clone());
                        };
                        if ui.button("🗺").clicked() {
                            targets_path.insert(k.to_owned(), WidgetPath { open: true, index: 0, history: path.clone(), latitude: 0.0, longitude: 0.0, altitude: 0.0, distance: 0.0, heading: 0.0 } );
                        };

                        ui.heading(k);
                        ui.color_edit_button_srgba(&mut path.color);
                        ComboBox::from_id_source(path.name.to_string())
                            .selected_text("")
                            .show_ui(ui, |ui| {
                                for marker in MarkerShape::all() {
                                    ui.selectable_value(
                                        &mut path.shape,
                                        marker,
                                        format!("{marker:?}"),
                                    );
                                }
                            });
                    });

                    for p in &path.history {
                        ui.label(p.name.clone());
                    }
                }
            }

            if let Some(k) = eviction_path {
                paths.remove(&k);
                if !paths.contains_key(displayed_path) {
                    *displayed_path = "Self".to_string();
                }
            }
        });
    }
}

impl WidgetTarget {
    pub fn display(&mut self, ctx: &egui::Context) {
        egui::Window::new(format!("{} - {}", self.target.container, self.target.name))
            .default_pos(Pos2::new(400.0, 800.0))
            .open(&mut self.open)
            .show(ctx, |ui| {
                egui::Grid::new("MainGrid").show(ui, |ui| {
                    ui.label("Latitute:");
                    ui.label(pretty(self.latitude));
                    ui.end_row();
                    ui.label("Longitude:");
                    ui.label(pretty(self.longitude));
                    ui.end_row();
                    ui.label("Altitude:");
                    ui.label(format!("{:.3}km", self.altitude));
                    ui.end_row();
                    ui.label("Distance:");
                    ui.label(format!("{:.3}km", self.distance));
                    ui.end_row();
                    ui.label("Heading:");
                    ui.label(pretty(self.heading));
                    ui.end_row();
                    ui.label("CIG Heading:");
                    ui.label(borked_cig_heading(self.heading));
                    ui.end_row();
                });
            });
    }
}

impl WidgetPath {
    pub fn display(&mut self, ctx: &egui::Context) {
        let current_point = &self.history.history[self.index];
        egui::Window::new(format!("Path - {}", self.history.name))
            .default_pos(Pos2::new(400.0, 800.0))
            .open(&mut self.open)
            .show(ctx, |ui| {
                egui::Grid::new("MainGrid").show(ui, |ui| {
                    ui.horizontal(|ui| {
if ui.button("⏴").clicked() & (self.index > 0) {
                                self.index -= 1;
                    };
                    if ui.button("⏵").clicked() & (self.index + 1 < self.history.history.len()) {
                                self.index += 1;
                    };
                    ui.heading(format!("{}/{}", self.index+1, self.history.history.len()));

                    });
                                        ui.heading(&current_point.name);

                                     ui.end_row();
                    ui.label("Latitute:");
                    ui.label(pretty(current_point.latitude));
                    ui.end_row();
                    ui.label("Longitude:");
                    ui.label(pretty(current_point.longitude));
                    ui.end_row();
                    ui.label("Altitude:");
                    ui.label(format!("{:.3}km", current_point.altitude));
                    ui.end_row();
                    ui.label("Distance:");
                    ui.label(format!("{:.3}km", self.distance));
                    ui.end_row();
                    ui.label("Heading:");
                    ui.label(pretty(self.heading));
                    ui.end_row();
                    ui.label("CIG Heading:");
                    ui.label(borked_cig_heading(self.heading));
                    ui.end_row();
                });
            });
    }
}

impl WidgetMap {
    pub fn display(
        &mut self,
        ctx: &egui::Context,
        targets: &WidgetTargets,
        paths: &HashMap<String, Path>,
    ) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Map");
            // TODO get map from scdatatools

            Plot::new("my_plot")
                // .min_size(Vec2::new(800.0,500.0))
                .view_aspect(1.0)
                // .data_aspect(2.0)
                .include_x(-PI)
                .include_x(PI)
                .include_y(PI / 2.0)
                .include_y(-PI / 2.0)
                .label_formatter(|name, value| {
                    if !name.is_empty() {
                        format!("{name}\n{}\n{}", pretty(value.y), pretty(value.x))
                    } else {
                        "".to_owned()
                    }
                })
                .show(ui, |plot_ui| {
                    for p in &targets.targets {
                        let c = [p.longitude, p.latitude];
                        plot_ui.points(
                            Points::new(c)
                                .name(p.target.name.clone())
                                .radius(3.0)
                                .shape(egui_plot::MarkerShape::Diamond),
                        );
                    }

                    // Construct & display all path !
                    for (k, path) in paths {
                        let mut point_path = Vec::new();
                        for p in &path.history {
                            let c = [
                                p.local_coordinates.longitude(),
                                p.local_coordinates.latitude(),
                            ];
                            point_path.push(c);
                            plot_ui.points(
                                Points::new(c)
                                    .name(p.name.clone())
                                    .radius(3.0)
                                    .color(path.color)
                                    .shape(path.shape),
                            );
                        }
                        plot_ui.line(Line::new(point_path).name(k).width(1.5).color(path.color));
                    }
                });
        });
    }
}
