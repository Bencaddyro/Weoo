use crate::{
    geolib::{Container, ProcessedPosition},
    iolib::{import_history, save_history, save_to_poi},
    mainlib::{WidgetMap, WidgetTarget, WidgetTargets, WidgetTopPosition},
};
use egui::{Color32, ComboBox, Context, Grid, Pos2, TextEdit, TopBottomPanel};
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
        database: &mut HashMap<String, Container>,
        index: &mut usize,
        position_history: &mut Vec<ProcessedPosition>,
        targets: &mut WidgetTargets,
        paths: &mut HashMap<String, (Color32, Vec<ProcessedPosition>)>,
    ) {
        let len = position_history.len();

        TopBottomPanel::top("top_panel").show(ctx, |ui| {
            // ui.columns(5, |columns| {
            ui.horizontal(|ui| {
                ui.vertical(|ui| {
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
                            paths.insert(
                                self.history_name.to_owned(),
                                (
                                    Color32::from_rgb(
                                        rand::thread_rng().gen(),
                                        rand::thread_rng().gen(),
                                        rand::thread_rng().gen(),
                                    ),
                                    import_history(&self.history_name),
                                ),
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
        position_history: &mut Vec<ProcessedPosition>,
        paths: &mut HashMap<String, (Color32, Vec<ProcessedPosition>)>,
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

            // clamp index if deletion
            let len = if position_history.is_empty() {
                0
            } else {
                position_history.len() - 1
            };
            *index = (*index).min(len);

            ui.heading("Paths");

            let mut eviction_path = None;

            for (k, (_c, v)) in paths.clone().iter() {
                ui.horizontal(|ui| {
                    if ui.button("❌").clicked() {
                        eviction_path = Some(k.clone());
                    };
                    ui.heading(format!("{k} path"));
                });

                for p in v {
                    ui.label(p.name.clone());
                }
            }

            if let Some(k) = eviction_path {
                paths.remove(&k);
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
        paths: &HashMap<String, (Color32, Vec<ProcessedPosition>)>,
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
                        println!("\n\n\n\n\n{p:?}");

                        let c = [p.longitude, p.latitude];
                        plot_ui.points(Points::new(c).name(p.target.name.clone()).radius(3.0));
                    }

                    // Construct & display history
                    let mut path = Vec::new();
                    for p in position_history {
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

                    //Now for all path !
                    for (k, (color, v)) in paths {
                        let mut path = Vec::new();
                        for p in v {
                            let c = [
                                p.local_coordinates.longitude(),
                                p.local_coordinates.latitude(),
                            ];
                            path.push(c);
                            plot_ui.points(
                                Points::new(c)
                                    .name(p.name.clone())
                                    .radius(3.0)
                                    .color(*color),
                            );
                        }
                        plot_ui.line(Line::new(path).name(k).width(1.5).color(*color));
                    }
                });
        });
    }
}
