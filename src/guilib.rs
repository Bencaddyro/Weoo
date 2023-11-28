use crate::{
    geolib::Container,
    mainlib::{WidgetMap, WidgetPosition, WidgetTarget, WidgetTargetSelection, ProcessedPosition}, iolib::{save_history, import_history},
};
use egui::{Align, Color32, Layout, Pos2};
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

impl WidgetPosition {
    pub fn display(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            egui::Grid::new("MainTopGrid").show(ui, |ui| {
                egui::Grid::new("SelfPosition").show(ui, |ui| {

                    ui.heading("Self Position");
                    ui.spinner();
                    ui.end_row();

                    if !self.position_history.is_empty() {
                    ui.label("Timestamp:");
                    ui.label(format!("{}", self.position_history[self.index].space_time_position.timestamp));
                    ui.end_row();
                    ui.label("Coordinates:");
                    ui.label(format!(
                        "x:{} y:{} z:{}",
                        self.position_history[self.index].space_time_position.coordinates.x,
                        self.position_history[self.index].space_time_position.coordinates.y,
                        self.position_history[self.index].space_time_position.coordinates.z
                    ));
                    ui.end_row();
                    ui.label("Container:");
                    ui.label(self.position_history[self.index].container.name.to_string());
                    ui.end_row();

                    ui.label("Latitute:");
                    ui.label(pretty(self.position_history[self.index].latitude));
                    ui.end_row();
                    ui.label("Longitude:");
                    ui.label(pretty(self.position_history[self.index].longitude));
                    ui.end_row();
                    ui.label("Altitude:");
                    ui.label(format!("{:.3}km", self.position_history[self.index].altitude));
                    ui.end_row();
                    };
                });

                ui.add(egui::Separator::default().vertical());

                ui.horizontal(|ui| {

                    if !self.position_history.is_empty() {

                        if ui.button("❌").clicked() {
                            self.eviction = Some(self.index);
                        };
                        if ui.button("⏴").clicked() & (self.index > 0) {
                            self.index -= 1;
                        };
                        if ui.button("⏵").clicked()
                            & (self.index + 1 < self.position_history.len())
                        {
                            self.index += 1;
                        };

                        ui.heading(format!("{}/{} : {}", self.index+1, self.position_history.len(), self.position_history[self.index].clone().name));
                        ui.add(egui::TextEdit::singleline(&mut self.name).hint_text("Custom Poi"));

                        if ui.button("Save").clicked() {
                            self.save_current_position();
                        };

                        ui.add(egui::TextEdit::singleline(&mut self.position_history[self.index].name).hint_text("No_name"));


                    } else {
                        ui.heading("No history 😕");

                    }



                });
                ui.end_row();


                ui.add(egui::Separator::default().vertical());
                if ui.button("Save History").clicked() {
                    save_history(&self.history_name, &self.position_history);
                };
                ui.add(egui::TextEdit::singleline(&mut self.history_name).hint_text("History_name"));

                ui.end_row();

                ui.add(egui::Separator::default().vertical());
                if ui.button("Import History").clicked() {
                    self.addition = import_history(&self.history_name);
                };
                ui.add(egui::TextEdit::singleline(&mut self.history_name).hint_text("History_name"));


            });
        });
    }
}

impl WidgetTargetSelection {
    pub fn display(
        &mut self,
        ctx: &egui::Context,
        database: &HashMap<String, Container>,
        // elapsed_time: f64,
        position: &ProcessedPosition,
    ) {
        egui::TopBottomPanel::bottom("bottom_panel").show(ctx, |ui| {
            ui.label("Select Target");

            egui::Grid::new("MainGrid").show(ui, |ui| {
                egui::ComboBox::from_label("Container")
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

                egui::ComboBox::from_label("Poi")
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

                if ui.button("Add Target").clicked()
                    & database.contains_key(&self.target_poi.container)
                {
                    self.targets.insert(
                        format!("{} - {}", self.target_container.name, self.target_poi.name),
                        WidgetTarget::new(self.target_poi.clone(), database),
                    );
                };

                ui.end_row();
            });
        });

        // Remove hidden targets
        self.targets.retain(|_, v| v.open);
        // Display targets windows
        for target in &mut self.targets.values_mut() {
            target.update(database, position);
            target.display(ctx);
        }
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
        database: &HashMap<String, Container>,
        position: &WidgetPosition,
    ) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.label("Select Target");

            egui::Grid::new("MainGrid").show(ui, |ui| {
                egui::ComboBox::from_label("Container")
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

                egui::ComboBox::from_label("Poi")
                    .selected_text(self.target_poi.name.clone())
                    .show_ui(ui, |ui| {
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
                    });

                if ui.button("Add Target").clicked()
                    & database.contains_key(&self.target_poi.container)
                {
                    self.targets.push((
                        self.target_poi.name.clone(),
                        [
                            self.target_poi.coordinates.longitude(),
                            self.target_poi.coordinates.latitude(),
                        ],
                    ));
                };

                ui.end_row();
            });
            ui.separator();
            ui.heading("Map");

            ui.with_layout(Layout::left_to_right(Align::TOP), |ui| {
                ui.vertical(|ui| {
                    ui.heading("Targets");

                    for (i, (name, _)) in self.targets.iter().enumerate() {
                        ui.horizontal(|ui| {
                            if ui.button("❌").clicked() {
                                self.eviction.push(i)
                            };
                            // if ui.button("⏶").clicked() { };
                            // if ui.button("⏷").clicked() { };

                            ui.label(name);
                        });
                    }
                    ui.heading("Self");
                    for p in &position.position_history {
                        ui.horizontal(|ui| {
                            if ui.button("❌").clicked() {
                                // self.eviction_self.push(i)
                            };
                            // if ui.button("⏶").clicked() { };
                            // if ui.button("⏷").clicked() { };

                            ui.label(p.name.clone());
                        });
                    }
                });

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
                        for (name, p) in self.targets.iter() {
                            // let y = (PI / 4.0 + p[1].to_radians() / 2.0).tan().abs().ln();
                            let c = [p[0], p[1]];
                            plot_ui.points(Points::new(c).name(name).radius(3.0));
                        }
                        let mut path = Vec::new();

                        for p in &position.position_history {
                            // let y = (PI / 4.0 + p[1].to_radians() / 2.0).tan().abs().ln();
                            let c = [p.local_coordinates.longitude(),p.local_coordinates.latitude()];
                            // let c = [p[0], p[1]];
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
        });
    }
}
