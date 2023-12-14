use crate::{
    geolib::{Container, Path},
    iolib::{import_history, save_history, save_to_poi},
    mainlib::{WidgetMap, WidgetPath, WidgetTarget, WidgetTargets, WidgetTopPosition},
};
use chrono::Duration;
use egui::{
    CollapsingHeader, Color32, ComboBox, Context, Grid, Pos2, RichText, TextEdit, TopBottomPanel,
    Ui,
};
use egui_plot::{Line, MarkerShape, Plot, Points};
use std::collections::{BTreeMap, HashMap};

pub fn pretty(a: f64) -> String {
    let degrees = a.to_degrees().trunc();
    let minutes = (a.to_degrees().fract() * 60.0).trunc().abs();
    let seconds = ((a.to_degrees().fract() * 60.0).fract() * 60.0)
        .trunc()
        .abs();
    format!("{degrees}° {minutes}’ {seconds}”")
}

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
                                    shape: MarkerShape::Circle,
                                    radius: 3.0,
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
            CollapsingHeader::new(RichText::new("Targets").heading())
                .default_open(true)
                .show(ui, |ui| {
                    let mut eviction = Vec::new();
                    for (i, e) in self.targets.iter_mut().enumerate() {
                        ui.horizontal(|ui| {
                            if ui.button("❌").clicked() {
                                eviction.push(i);
                            };
                            if ui.button("👁").clicked() {
                                e.open = !e.open;
                            };

                            ui.label(&e.target.name);
                        });

                        e.display(ctx);
                    }
                    for i in eviction {
                        self.targets.remove(i);
                    }
                });

            ui.heading("Paths");

            let mut eviction_path = None;
            for (k, path) in paths.iter_mut() {
                eviction_path = display_path(ui, path, targets_path);

                if path.history.is_empty() & (k != "Self") {
                    eviction_path = Some(k.to_string());
                }
            }

            // clamp index if deletion
            let len = if paths.get_mut(displayed_path).unwrap().history.is_empty() {
                0
            } else {
                paths.get_mut(displayed_path).unwrap().history.len() - 1
            };
            *index = (*index).min(len);

            if let Some(k) = eviction_path {
                paths.remove(&k);
            }
        });
    }
}

pub fn display_path(
    ui: &mut Ui,
    path: &mut Path,
    targets_path: &mut HashMap<String, WidgetPath>,
) -> Option<String> {
    let mut eviction = None;
    let mut up = None;
    let mut down = None;
    let len = path.history.len();
    let k = path.name.to_string();
    let mut eviction_path = None;

    let id = ui.make_persistent_id(k.to_string());
    egui::collapsing_header::CollapsingState::load_with_default_open(ui.ctx(), id, false)
        .show_header(ui, |ui| {
            ui.spacing_mut().item_spacing = egui::vec2(1.0, 1.0);

            ui.heading(k.to_string());

            if ui.button("❌").clicked() & (path.name != "Self") {
                eviction_path = Some(k.to_string());
            };
            if ui.button("🗺").clicked() {
                targets_path.insert(
                    k.to_string(),
                    WidgetPath {
                        open: true,
                        index: 0,
                        history: k.to_string(),
                        latitude: 0.0,
                        longitude: 0.0,
                        altitude: 0.0,
                        distance: 0.0,
                        heading: 0.0,
                        duration: Duration::zero(),
                        length: 0.0,
                    },
                );
            };

            ui.color_edit_button_srgba(&mut path.color);

            ui.add(
                egui::DragValue::new(&mut path.radius)
                    .speed(0.1)
                    .clamp_range(0..=10),
            );
        })
        .body(|ui| {
            for (i, p) in path.history.iter().enumerate() {
                ui.spacing_mut().item_spacing = egui::vec2(1.0, 1.0);
                ui.horizontal(|ui| {
                    ui.menu_button("⚙", |ui| {
                        ui.horizontal(|ui| {
                            ui.spacing_mut().item_spacing = egui::vec2(1.0, 1.0);
                            if ui.button("❌").clicked() {
                                eviction = Some(i);
                                ui.close_menu();
                            };
                            if ui.button("⌖").clicked() {
                                targets_path.insert(
                                    k.to_string(),
                                    WidgetPath {
                                        open: true,
                                        index: i,
                                        history: k.to_string(),
                                        latitude: 0.0,
                                        longitude: 0.0,
                                        altitude: 0.0,
                                        distance: 0.0,
                                        heading: 0.0,
                                        duration: Duration::zero(),
                                        length: 0.0,
                                    },
                                );
                                ui.close_menu();
                            };
                            // ui.color_edit_button_srgba(&mut path.color);
                        });
                    });

                    if ui.button("⏶").clicked() & (len > 1) {
                        up = Some(i);
                    };
                    if ui.button("⏷").clicked() & (len > 1) {
                        down = Some(i);
                    };
                    ui.label(&p.name);
                });
            }
        });
    if let Some(i) = eviction {
        path.history.remove(i);
    } else if let Some(i) = up {
        let point = path.history.remove(i);
        path.history.insert(i.max(1) - 1, point)
    } else if let Some(i) = down {
        let point = path.history.remove(i);
        path.history.insert(i.min(len - 2) + 1, point)
    }

    eviction_path
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
    pub fn display(&mut self, ctx: &egui::Context, paths: &HashMap<String, Path>) {
        let path = paths.get(&self.history).unwrap();

        let current_point = &path.history[self.index];
        egui::Window::new(format!("Path - {}", self.history))
            .default_pos(Pos2::new(400.0, 800.0))
            .open(&mut self.open)
            .show(ctx, |ui| {
                egui::Grid::new("MainGrid").show(ui, |ui| {
                    ui.horizontal(|ui| {
                        if ui.button("⏴").clicked() & (self.index > 0) {
                            self.index -= 1;
                        };
                        if ui.button("⏵").clicked() & (self.index + 1 < path.history.len()) {
                            self.index += 1;
                        };
                        ui.heading(format!("{}/{}", self.index + 1, path.history.len()));
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
                .data_aspect(1.0)
                // .x_axis_formatter(legend)
                // .x_grid_spacer(grid)
                // .y_axis_formatter(legend)
                // .y_grid_spacer(grid)
                .include_x(-180)
                .include_x(180)
                .include_y(90)
                .include_y(-90)
                .label_formatter(|name, value| {
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
                    for p in &targets.targets {
                        let c = [p.longitude.to_degrees(), p.latitude.to_degrees()];
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
                                p.local_coordinates.longitude().to_degrees(),
                                p.local_coordinates.latitude().to_degrees(),
                            ];
                            point_path.push(c);
                            plot_ui.points(
                                Points::new(c)
                                    .name(p.name.clone())
                                    .radius(path.radius)
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
