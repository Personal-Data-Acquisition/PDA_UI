mod utils;
mod line_drawing;

use egui::*;
use egui_plot::{Legend, Line, Plot, PlotPoints};
use wasm_bindgen::prelude::*;
use std::collections::HashMap;
use log::debug;
use wasm_bindgen_futures::spawn_local;
use serde::{Deserialize, Serialize};
use walkers::{Tiles, Map, MapMemory, Position, TilesManager, HttpOptions};
use crate::line_drawing::GpsLine;
use crate::utils::PollableValue;

const TITLE: &str = "Personal Data Acquisition";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Provider {
    OpenStreetMap,
    Geoportal,
    MapboxStreets,
    MapboxSatellite,
    LocalTiles,
}

#[wasm_bindgen]
pub fn main() {
    eframe::WebLogger::init(log::LevelFilter::Debug).ok();
    let web_options = eframe::WebOptions::default();

    spawn_local(async {
        eframe::WebRunner::new()
            .start(
                TITLE,
                web_options,
                Box::new(|cc| Box::new(MyApp::new(cc.egui_ctx.clone()))),
            )
            .await
            .expect("failed to start eframe");
    });
}

#[wasm_bindgen]
pub struct MyApp {
    open_panel: Panel,
    home_panel: HomePanel,
    log_panel: LogPanel,
    config_panel: ConfigPanel,
    map_memory: MapMemory,
    providers: HashMap<Provider, Box<dyn TilesManager + Send>>,
    gps_points: PollableValue<Vec<[f64; 2]>>,
}

impl Default for MyApp {
    fn default() -> Self {
        Self {
            // tiles: Tiles::new(OpenStreetMap, Context::default()),
            map_memory: MapMemory::default(),
            open_panel: Panel::default(),
            home_panel: HomePanel::default(),
            log_panel: LogPanel::default(),
            config_panel: ConfigPanel::default(),
            providers: providers(Context::default()),
            gps_points: PollableValue::new(
                Default::default(),
                poll_promise::Promise::spawn_local(async {
                    GpsLine::req_points().await
                })
            ),
        }
    }
}

impl MyApp {
    fn new(egui_ctx: Context) -> Self {
        Self {
            map_memory: MapMemory::default(),
            open_panel: Panel::default(),
            home_panel: HomePanel::default(),
            log_panel: LogPanel::default(),
            config_panel: ConfigPanel::default(),
            providers: providers(egui_ctx),
            gps_points: PollableValue::new(
                Default::default(),
                poll_promise::Promise::spawn_local(async {
                    GpsLine::req_points().await
                })
            ),
        }
    }
}

fn providers(egui_ctx: Context) -> HashMap<Provider, Box<dyn TilesManager + Send>> {
    let mut providers: HashMap<Provider, Box<dyn TilesManager + Send>> = HashMap::default();

    providers.insert(
        Provider::OpenStreetMap,
        Box::new(Tiles::with_options(
            walkers::sources::OpenStreetMap,
            http_options(),
            egui_ctx.to_owned(),
        )),
    );  

    providers
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.selectable_value(&mut self.open_panel, Panel::Home, "Home");
                ui.selectable_value(&mut self.open_panel, Panel::Log, "Log");
                ui.selectable_value(&mut self.open_panel, Panel::Config, "Config");
            });
            ui.separator();

            match self.open_panel {
                Panel::Home => {      
                    ScrollArea::vertical()
                        .auto_shrink([false, false])
                        .stick_to_bottom(false)
                        .show(ui, |ui| {
                            self.home_panel.ui(ui);

                            let tiles = self.providers.get_mut(&Provider::OpenStreetMap).unwrap().as_mut();

                            let mut map = Map::new(
                                Some(tiles),
                                &mut self.map_memory,
                                Position::from_lat_lon(44.56203897286608, -123.28196905234289));

                            if self.gps_points.poll() {
                                map = map.with_plugin(GpsLine::new(self.gps_points.value.clone()));
                            }

                            ui.add_sized([ui.available_width(), 600.0], map);
                            zoom(ui, &mut self.map_memory);
                        });
                }
                Panel::Log => {
                    self.log_panel.ui(ui);
                }
                Panel::Config => {
                    self.config_panel.ui(ui);
                }
            }
        });
    }
}

fn http_options() -> HttpOptions {
    HttpOptions {
        cache: if std::env::var("NO_HTTP_CACHE").is_ok() {
            None
        } else {
            Some(".cache".into())
        },
    }
}

#[derive(PartialEq, Eq)]
#[wasm_bindgen]
pub enum Panel {
    Home,
    Log,
    Config,
}

impl Default for Panel {
    fn default() -> Self {
        Self::Home
    }
}

pub async fn send_update(body: &HashMap<&str, &str>, url: &str) {
    let client = reqwest_wasm::Client::new();
    let res = client.post(url)
        .json(body)
        .send()
        .await.expect("no response")
        .text()
        .await;
    debug!("res: {:?}", res);
}

// Panels ---------------------------------------

#[wasm_bindgen]
pub struct HomePanel {
    is_recording: bool,
    accel_x: PollableValue<Vec<[f64; 2]>>,
    accel_y: PollableValue<Vec<[f64; 2]>>,
    accel_z: PollableValue<Vec<[f64; 2]>>,
}

impl Default for HomePanel {
    fn default() -> Self {
        Self {
            is_recording: false,
            accel_x: PollableValue::new(
                Default::default(), 
                poll_promise::Promise::spawn_local(async {
                    HomePanel::req_data_latest("acceleration_x").await
                })
            ),
            accel_y: PollableValue::new(
                Default::default(), 
                poll_promise::Promise::spawn_local(async {
                    HomePanel::req_data_latest("acceleration_y").await
                })
            ),
            accel_z: PollableValue::new(
                Default::default(), 
                poll_promise::Promise::spawn_local(async {
                    HomePanel::req_data_latest("acceleration_z").await
                })
            ),
        }
    }
}

impl HomePanel {
    fn ui(&mut self, ui: &mut Ui) {
        let plot_accel_x = Plot::new("Acceleration X")
            .legend(Legend::default())
            .height(200.0);
        let plot_accel_y = Plot::new("Acceleration Y")
            .legend(Legend::default())
            .height(200.0);
        let plot_accel_z = Plot::new("Acceleration Z")
            .legend(Legend::default())
            .height(200.0);

        if self.accel_x.poll() {
            let line = Line::new(PlotPoints::from(self.accel_x.get_value().clone())).name("Acceleration X");
            plot_accel_x.show(ui, |plot_ui| {
                plot_ui.line(line);
            });
        }

        if self.accel_y.poll() {
            let line = Line::new(PlotPoints::from(self.accel_y.get_value().clone())).name("Acceleration Y");
            plot_accel_y.show(ui, |plot_ui| {
                plot_ui.line(line);
            });
        }

        if self.accel_z.poll() {
            let line = Line::new(PlotPoints::from(self.accel_z.get_value().clone())).name("Acceleration Z");
            plot_accel_z.show(ui, |plot_ui| {
                plot_ui.line(line);
            });
        }

        ui.horizontal(|ui| {
            if !self.is_recording {
                if ui.button("Record").clicked() {
                    self.is_recording = true;
                    let mut body = HashMap::new();
                    body.insert("isRecording", "true");
                    spawn_local(async move {
                        send_update(&body, "http://127.0.0.1:8000/update").await;
                    })
                }
            } else {
                if ui.button("Stop").clicked() {
                    self.is_recording = false;
                    let mut body = HashMap::new();
                    body.insert("isRecording", "false");
                    spawn_local(async move {
                        send_update(&body, "http://127.0.0.1:8000/update").await;
                    })
                }
            }
        });
    }

    /// Requests data of type `Option<Vec<[f64; 2]>>` from the server
    async fn req_data_latest(param: &str) -> Option<Vec<[f64; 2]>> {
        let client = reqwest_wasm::Client::new();
        let res = match client.get("http://127.0.0.1:8000/req/data/latest/".to_owned() + param).send().await {
            Err(why) => {
                debug!("failed to get: {}", why);
                return None;
            },
            Ok(result) => {
                result
            },
        };
        return match res.json::<Vec<[f64; 2]>>().await {
            Err(why) => {
                debug!("failed to parse json: {},", why);
                None
            },
            Ok(result) => {
                Some(result)
            }
        }
    }
}

#[wasm_bindgen]
pub struct LogPanel {
    accel: PollableValue<Vec<[String; 5]>>,
}

impl Default for LogPanel {
    fn default() -> Self {
        Self {
            accel: PollableValue::new(
                Default::default(), 
                poll_promise::Promise::spawn_local(async {
                    LogPanel::req_data_full("acceleration").await
                })
            ),
        }
    }
}

impl LogPanel {
    fn ui(&mut self, ui: &mut Ui) {
        use egui_extras::{Column, TableBuilder};

        let table = TableBuilder::new(ui)
            .striped(true)
            .resizable(true)
            .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
            .column(Column::auto())
            .column(Column::auto())
            .column(Column::auto())
            .column(Column::auto())
            .column(Column::auto())
            .min_scrolled_height(0.0);

        table
            .header(20.0, |mut header| {
                header.col(|ui| {
                    ui.strong("Row");
                });
                header.col(|ui| {
                    ui.strong("Time");
                });
                header.col(|ui| {
                    ui.strong("Acceleration X");
                });
                header.col(|ui| {
                    ui.strong("Acceleration Y");
                });
                header.col(|ui| {
                    ui.strong("Acceleration Z");
                });
            })
            .body(|mut body| {
                let row_height = 18.0;

                if self.accel.poll() {
                    let table_data = self.accel.get_value().clone();
                    for entry in table_data {
                        body.row(row_height, |mut row| {
                            row.col(|ui| {
                                ui.label(entry[0].to_string());
                            });
                            row.col(|ui| {
                                ui.add(
                                    egui::Label::new(entry[1].to_string()).wrap(false),
                                );
                            });
                            row.col(|ui| {
                                ui.add(
                                    egui::Label::new(entry[2].to_string()).wrap(false),
                                );
                            });
                            row.col(|ui| {
                                ui.add(
                                    egui::Label::new(entry[3].to_string()).wrap(false),
                                );
                            });
                            row.col(|ui| {
                                ui.add(
                                    egui::Label::new(entry[4].to_string()).wrap(false),
                                );
                            });
                        });
                    }
                }
            });
    }

    /// Requests data of type `Option<Vec<[String; 5]>>` from the server
    async fn req_data_full(param: &str) -> Option<Vec<[String; 5]>> {
        let client = reqwest_wasm::Client::new();
        let res = match client.get("http://127.0.0.1:8000/req/data/full/".to_owned() + param).send().await {
            Err(why) => {
                debug!("failed to get: {}", why);
                return None;
            },
            Ok(result) => {
                result
            },
        };
        return match res.json::<Vec<[String; 5]>>().await {
            Err(why) => {
                debug!("failed to parse json: {},", why);
                None
            },
            Ok(result) => {
                Some(result)
            }
        }
    }
}

#[wasm_bindgen]
#[derive(Copy, Clone, Deserialize, Serialize)]
pub struct Config {
    temp_enabled: bool,
    temp_sensitivity: f32,
    accel_enabled: bool,
    accel_sensitivity: f32,
}

impl Config {
    pub fn new(
        temp_enabled: bool, 
        temp_sensitivity: f32, 
        accel_enabled: bool, 
        accel_sensitivity: f32) -> Self {
            
        Self {
            temp_enabled,
            temp_sensitivity,
            accel_enabled,
            accel_sensitivity,
        }
    }
}

#[wasm_bindgen]
pub struct ConfigPanel {
    settings_promise: poll_promise::Promise<Option<String>>,
    config_received: bool,
    config: Config,
}

impl Default for ConfigPanel {
    fn default() -> Self {
        Self {
            settings_promise: poll_promise::Promise::spawn_local(async {
                ConfigPanel::req_settings().await
            }),
            config_received: false,
            config: Config::new(true, 8.0, true, 8.0),
        }
    }
}

impl ConfigPanel {
    fn ui(&mut self, ui: &mut Ui) {
        egui::TopBottomPanel::top("temp_panel")
            .show_inside(ui, |ui| {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    ui.vertical_centered(|ui| {
                        ui.heading("Temperature Sensor");
                    });
                    ui.checkbox(&mut self.config.temp_enabled, "Enabled");
                    ui.add_enabled_ui(self.config.temp_enabled, |ui| {
                        ui.horizontal(|ui| {
                            ui.add(egui::DragValue::
                                   new(&mut self.config.temp_sensitivity).speed(0.1));
                            ui.add(egui::Label::new("Sensitivity"));
                        });
                    });
                });
            });
        egui::TopBottomPanel::top("accel_panel")
            .show_inside(ui, |ui| {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    ui.vertical_centered(|ui| {
                        ui.heading("Accelerometer");
                    });
                    ui.checkbox(&mut self.config.accel_enabled, "Enabled");
                    ui.add_enabled_ui(self.config.accel_enabled, |ui| {
                        ui.horizontal(|ui| {
                            ui.add(egui::DragValue::
                                   new(&mut self.config.accel_sensitivity).speed(0.1));
                            ui.add(egui::Label::new("Sensitivity"));
                        });
                    });
                });
            });
        ui.horizontal(|ui| {
            // this button will send config to server
            if ui.button("Save").clicked() {
                let config = self.config.clone();
                spawn_local(async move {
                    ConfigPanel::send_settings_update(config).await;
                });
            }
        });
        // Poll config until ready
        if !self.config_received {
            if let Some(result) = self.settings_promise.ready() {
                if let Some(json) = result {
                    self.config = serde_json::from_str(json.as_str()).unwrap();
                    self.config_received = true;
                } else {
                    debug!("Result error")
                }
            }
        }
    }

    /// Sends config to sever as JSON
    async fn send_settings_update(config: Config) {
        let client = reqwest_wasm::Client::new();
        let res = client.post("http://127.0.0.1:8000/update/settings")
            .json(&serde_json::to_string(&config).expect("couldn't serialize"))
            .send()
            .await.expect("no response")
            .text()
            .await;
        debug!("res: {:?}", res);
    }
    
    /// Requests config from server, and pack it into a promise
    async fn req_settings() -> Option<String> {
        let client = reqwest_wasm::Client::new();
        let res = match client.get("http://127.0.0.1:8000/req/settings").send().await {
            Err(why) => {
                debug!("failed to get: {}", why);
                return None;
            },
            Ok(result) => {
                result
            },
        };
        return match res.json().await {
            Err(why) => {
                debug!("failed parse json: {}", why);
                return None;
            },
            Ok(result) => {
                Some(result)
            }
        }
    }
}

pub fn zoom(ui: &Ui, map_memory: &mut MapMemory) {
    Window::new("Map")
        .collapsible(false)
        .resizable(false)
        .title_bar(false)
        .anchor(Align2::LEFT_BOTTOM, [10., -10.])
        .show(ui.ctx(), |ui| {
            ui.horizontal(|ui| {
                if ui.button(RichText::new("➕").heading()).clicked() {
                    let _ = map_memory.zoom_in();
                }

                if ui.button(RichText::new("➖").heading()).clicked() {
                    let _ = map_memory.zoom_out();
                }
            });
        });
}