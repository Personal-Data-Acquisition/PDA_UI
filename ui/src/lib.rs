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
// all of the auto-refreshing data on the home panel
// displayed from first to last
// TODO: maybe try something better?
const HOME_PANEL_KEYS: [&str; 3] = [
    "acceleration_x",
    "acceleration_y",
    "acceleration_z",
];

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
                }),
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
                            self.home_panel.ui(ui, &self.config_panel.config);

                            let tiles = self.providers.get_mut(&Provider::OpenStreetMap).unwrap().as_mut();

                            let mut map = Map::new(
                                Some(tiles),
                                &mut self.map_memory,
                                Position::from_lat_lon(44.56203897286608, -123.28196905234289));

                            if let Some(res) = self.gps_points.poll() {
                                map = map.with_plugin(GpsLine::new(res));
                            }

                            ui.add_sized([ui.available_width(), 600.0], map);
                            zoom(ui, &mut self.map_memory);
                        });
                }
                Panel::Log => {
                    self.log_panel.ui(ui, &self.config_panel.config);
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
    data: HomePanelData,
}

/// refers to all of the auto-refreshing data on the home panel
struct HomePanelData {
    time: u16,
    pub data: HashMap<String, PollableValue<Vec<[f64; 2]>>>,
}

impl HomePanelData {
    fn new(defaults: HashMap<String, Option<Vec<[f64; 2]>>>) -> Self {
        let mut data: HashMap<String, PollableValue<Vec<[f64; 2]>>> = HashMap::new();
        for key in HOME_PANEL_KEYS {
            let default = defaults.get(key).expect("missing key in HomePanelData defaults").clone();
            data.insert(key.to_string(), PollableValue::new(
                default,
                poll_promise::Promise::spawn_local(async move {
                    HomePanel::req_data_latest(key).await   
                })  
            ));
        }
        Self {
            data,
            time: 0,
        }
    }
}

impl Default for HomePanel {
    fn default() -> Self {
        let mut defaults: HashMap<String, std::option::Option<std::vec::Vec<[f64; 2]>>> = HashMap::new();
        for key in HOME_PANEL_KEYS {
            defaults.insert(key.to_string(), None);
        }
        Self {
            is_recording: false,
            data: HomePanelData::new(defaults),
        }
    }
}

impl HomePanel {
    fn ui(&mut self, ui: &mut Ui, config: &Config) {
        let mut ready_count = 0;
        // graphs showing auto-refreshing data
        // keys should be provided in HOME_PANEL_KEYS
        for key in HOME_PANEL_KEYS {
            let val: &mut PollableValue<std::vec::Vec<[f64; 2]>> = self.data.data.get_mut(key).expect("missing key in HomePanelData");
            if let Some(res) = val.poll() {
                ready_count += 1;
                let plot = Plot::new(key)
                    .legend(Legend::default())
                    .height(200.0);
                let line = Line::new(PlotPoints::from(res)).name(key);
                plot.show(ui, |plot_ui| {
                    plot_ui.line(line);
                });
            }
        }
        // ui.heading(format!("ready count: {ready_count}"));
        // ui.heading(format!("time: {t}", t=self.data.time));
        // if all have been recieved, count up to refresh_time to refresh
        if ready_count == HOME_PANEL_KEYS.len() {
            self.data.time += 1;
            if self.data.time == (config.refresh_time * 60.0) as u16 {
                let mut defaults: HashMap<String, std::option::Option<Vec<[f64; 2]>>> = HashMap::new();
                for key in HOME_PANEL_KEYS {
                    let val: &mut PollableValue<Vec<[f64; 2]>> = self.data.data.get_mut(key)
                        .expect("missing key in HomePanelData");
                    defaults.insert(key.to_string(), val.poll());
                }
                self.data = HomePanelData::new(
                    defaults
                )
            }
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

struct LogPanelData {
    data: PollableValue<Vec<[String; 5]>>,   
    time: u16,
}

impl LogPanelData {
    fn new(default: Option<Vec<[String; 5]>>) -> Self {
        Self {
            data: PollableValue::new(
                default,
                poll_promise::Promise::spawn_local(async move {
                    LogPanel::req_data_full("acceleration").await // TEMP, TODO genericize this
                })),
            time: 0,
        }
    }
}

#[wasm_bindgen]
pub struct LogPanel {
    data: LogPanelData,
}

impl Default for LogPanel {
    fn default() -> Self {
        Self {
            data: LogPanelData::new(None)
        }
    }
}

impl LogPanel {
    fn ui(&mut self, ui: &mut Ui, config: &Config) {
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

                if let Some(table_data) = self.data.data.poll() {
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
                    // update timer
                    self.data.time += 1;
                    if self.data.time == (config.refresh_time * 60.0) as u16 {
                        self.data = LogPanelData::new(
                            self.data.data.value.clone()
                        )
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
    refresh_time: f32, // seconds
}

impl Config {
    pub fn new(
        temp_enabled: bool, 
        temp_sensitivity: f32, 
        accel_enabled: bool, 
        accel_sensitivity: f32,
        refresh_time: f32) -> Self {
            
        Self {
            temp_enabled,
            temp_sensitivity,
            accel_enabled,
            accel_sensitivity,
            refresh_time
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
            config: Config::new(
                true, 
                8.0, 
                true, 
                8.0,
                1.0),
        }
    }
}

impl ConfigPanel {
    fn ui(&mut self, ui: &mut Ui) {
        egui::TopBottomPanel::top("general_panel")
            .show_inside(ui, |ui| {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    ui.vertical_centered(|ui| {
                        ui.heading("General");
                    });
                    ui.horizontal(|ui| {
                        ui.add(egui::DragValue::
                                new(&mut self.config.refresh_time).speed(0.1)
                                .clamp_range(0..=120));
                        ui.add(egui::Label::new("Refresh Delay (seconds)"));
                    });
                });
            });
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
                                   new(&mut self.config.temp_sensitivity).speed(0.1)
                                   .clamp_range(0..=120));
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
                                   new(&mut self.config.accel_sensitivity).speed(0.1)
                                   .clamp_range(0..=120));
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