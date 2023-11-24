use crate::{
    geolib::Container,
    mainlib::{WidgetMap, WidgetPoi, WidgetPosition, WidgetTarget, WidgetTargetSelection},
};
use egui::{Align, Layout};
use egui_plot::{Plot, Points};
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

                    ui.label("Timestamp:");
                    ui.label(format!("{}", self.space_time_position.timestamp));
                    ui.end_row();
                    ui.label("Coordinates:");
                    ui.label(format!(
                        "x:{} y:{} z:{}",
                        self.space_time_position.coordinates.x,
                        self.space_time_position.coordinates.y,
                        self.space_time_position.coordinates.z
                    ));
                    ui.end_row();
                    ui.label("Container:");
                    ui.label(self.container.name.to_string());
                    ui.end_row();

                    ui.label("Latitute:");
                    ui.label(pretty(self.latitude));
                    ui.end_row();
                    ui.label("Longitude:");
                    ui.label(pretty(self.longitude));
                    ui.end_row();
                    ui.label("Altitude:");
                    ui.label(format!("{:.3}km", self.altitude));
                    ui.end_row();
                });

                ui.add(egui::Separator::default().vertical());

                ui.end_row();
            });
        });
    }
}

impl WidgetTargetSelection {
    pub fn display(
        &mut self,
        ctx: &egui::Context,
        database: &HashMap<String, Container>,
        elapsed_time: f64,
        position: &WidgetPosition,
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
            target.update(database, elapsed_time, position);
            target.display(ctx);
        }
    }
}

impl WidgetTarget {
    pub fn display(&mut self, ctx: &egui::Context) {
        egui::Window::new(format!("{} - {}", self.target.container, self.target.name))
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

impl WidgetPoi {
    pub fn display(&mut self, ctx: &egui::Context) {
        egui::Window::new("Save Poi")
            // .open(&mut self.open)
            .show(ctx, |ui| {
                ui.label("Name:");

                ui.add(egui::TextEdit::singleline(&mut self.name).hint_text("Custom Poi"));
                if ui.button("Save").clicked() {
                    self.save_current_position();
                };
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
        egui::Window::new("Map").show(ctx, |ui| {
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

                    for (i,(name,_)) in self.targets.iter().enumerate() {
                        ui.horizontal(|ui| {

                            if ui.button("❌").clicked() { self.eviction.push(i) };
                            // if ui.button("⏶").clicked() { };
                            // if ui.button("⏷").clicked() { };

                            ui.label(name);
                        });
                    }
                    ui.heading("Self");
                    for (i,(name,_)) in self.travel.iter().enumerate() {
                        ui.horizontal(|ui| {

                            if ui.button("❌").clicked() { self.eviction_self.push(i) };
                            // if ui.button("⏶").clicked() { };
                            // if ui.button("⏷").clicked() { };

                            ui.label(name);
                        });
                    }

                });


            // Trace of different points based on history module ?

            // plot satelite screen based on coordinates found on lidar

            // TODO : how to scale screen shot ? 1920*1080 = q lat + j long -> mercator deformation :explosion_head:

            // screenshot : head to 0° / pitch 0°
            // Clever way : get current heading by diff position betweenscreenshot

            Plot::new("my_plot")
                // .view_aspect(2.0)
                // .data_aspect(2.0)
                .include_x(-PI)
                .include_x(PI)
                .include_y(PI / 2.0)
                .include_y(-PI / 2.0)
                .label_formatter(|name, value| {
                    if !name.is_empty() {
                        format!("{name}\n{}\n{}", pretty(value.x),pretty(value.y))
                    } else {
                        "".to_owned()
                    }
                })
                .show(ui, |plot_ui| {
                    for (name,p) in self.targets.iter() {
                        // let y = (PI / 4.0 + p[1].to_radians() / 2.0).tan().abs().ln();
                        let c = [p[0],p[1]];
                        plot_ui.points(Points::new(c).name(name).radius(3.0));
                    }
                    for (name,p) in self.travel.iter() {
                        // let y = (PI / 4.0 + p[1].to_radians() / 2.0).tan().abs().ln();
                        let c = [p[0],p[1]];
                        plot_ui.points(Points::new(c).name(name).radius(3.0));
                    }

                    // plot_ui.points(Points::new([position.local_coordinates.longitude(),position.local_coordinates.latitude()]).name("Position"));


                });
            });
        });
    }
}
