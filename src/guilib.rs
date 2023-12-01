use crate::{
    geolib::{Container, ProcessedPosition},
    iolib::{import_history, save_history},
    mainlib::{WidgetMap, WidgetTarget, WidgetTargets, WidgetTopPosition},
};
use egui::{
    Align, Color32, ComboBox, Context, Grid, Layout, Pos2, Separator, TextEdit, TopBottomPanel,
};
use egui_plot::{Line, Plot, Points};
use std::{collections::HashMap, f64::consts::PI};

pub fn pretty(a: f64) -> String {
    let degrees = a.to_degrees().trunc();
    let minutes = (a.to_degrees().fract() * 60.0).trunc().abs();
    let seconds = ((a.to_degrees().fract() * 60.0).fract() * 60.0)
        .trunc()
        .abs();
    format!("{degrees}° {minutes}’ {seconds}”")
}

// ui.label("Debug:");
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
        database: &HashMap<String, Container>,
        index: &mut usize,
        position_history: &mut Vec<ProcessedPosition>,
        targets: &mut WidgetTargets,
        paths: &mut HashMap<String, Vec<ProcessedPosition>>,
    ) {
        let len = position_history.len();

        TopBottomPanel::top("top_panel").show(ctx, |ui| {
            // ui.columns(5, |columns| {
            ui.horizontal(|ui| {
                ui.vertical(|ui| {
                    // let ui = &mut columns[0];
                    // Current position
                    ui.horizontal(|ui| {
                        ui.heading("Self Position");
                        ui.spinner();
                    });

                    if let Some(position) = position_history.get_mut(*index) {
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
                            ui.label(position.container.name.to_string());
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
                        ui.heading("Path History");

                        if let Some(position) = position_history.get_mut(*index) {
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
                                ui.add(
                                    TextEdit::singleline(&mut position.name).hint_text("No_name"),
                                );

                                if ui.button("Save as POI").clicked() {
                                    // self.save_current_position();
                                };

                                ui.end_row();
                            });
                        } else {
                            ui.heading("No Position 😕");
                        };
                        if eviction {
                            position_history.remove(*index);
                        }

                        // ui.separator(); // BUG fill entire right panel
                        ui.label("---------------------------");
                        ui.vertical(|ui| {
                            ui.heading("Path I/O");

                            if ui.button("Export Path").clicked() {
                                save_history(&self.history_name, position_history);
                            };
                            if ui.button("Import Path").clicked() {
                                paths.insert(self.history_name.to_owned(), import_history(&self.history_name));
                            };
                            ui.add(
                                TextEdit::singleline(&mut self.history_name).hint_text("Path Name"),
                            );
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
                                // format!("{} - {}", self.target_container.name, self.target_poi.name),
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
        position_history: &mut Vec<ProcessedPosition>,
    ) {
        egui::SidePanel::left("my_left_panel").show(ctx, |ui| {
            ui.heading("Targets");

            let mut eviction = Vec::new();
            for (i, e) in self.targets.iter_mut().enumerate() {
                ui.horizontal(|ui| {
                    if ui.button("❌").clicked() {
                        eviction.push(i);
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

            for (i, p) in position_history.iter().enumerate() {
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
                position_history.remove(i);
            }

            // clamp index cause deletion
            let len = if position_history.is_empty() {
                0
            } else {
                position_history.len() - 1
            };
            *index = (*index).min(len);

            // Remove hidden targets
            // self.targets.retain(|_, v| v.open);
            // Display targets windows
            // for target in &mut self.targets.values_mut() {
            //     target.update(database, position);
            //     target.display(ctx);
            // }
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
                    // ui.label("Delta:");
                    // ui.label(format!("{:?}", self.delta_distance));
                    // ui.end_row();
                });
            });
    }
}

impl WidgetMap {
    pub fn display(
        &mut self,
        ctx: &egui::Context,
        position_history: &mut Vec<ProcessedPosition>,
        targets: &WidgetTargets,
    ) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Map");

            // plot satelite screen based on coordinates found on lidar
            // TODO : how to scale screen shot ? 1920*1080 = q lat + j long -> mercator deformation :explosion_head:

            // screenshot : head to 0° / pitch 0°
            // Clever way : get current heading by diff position betweenscreenshot

            Plot::new("my_plot")
                // .min_size(Vec2::new(800.0,500.0))
                // .view_aspect(2.0)
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
                        // let y = (PI / 4.0 + p[1].to_radians() / 2.0).tan().abs().ln();

                        let c = [p.target.longitude.unwrap(), p.target.latitude.unwrap()];
                        plot_ui.points(Points::new(c).name(p.target.name.clone()).radius(3.0));
                    }
                    let mut path = Vec::new();

                    for p in position_history {
                        // let y = (PI / 4.0 + p[1].to_radians() / 2.0).tan().abs().ln();
                        let c = [
                            p.local_coordinates.longitude(),
                            p.local_coordinates.latitude(),
                        ];
                        path.push(c);
                        plot_ui.points(
                            Points::new(c)
                                .name(p.name.clone())
                                .radius(3.0)
                                .color(Color32::DARK_GRAY),
                        );
                    }

                    plot_ui.line(
                        Line::new(path)
                            .name("Self")
                            .width(1.5)
                            .color(Color32::DARK_GRAY),
                    );
                });
        });
    }
}
