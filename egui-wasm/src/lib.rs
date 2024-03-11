mod utils;

use egui::*;
use egui_plot::{Legend, Line, Plot, PlotPoints};
use wasm_bindgen::prelude::*;
use std::collections::HashMap;
use log::debug;
use wasm_bindgen_futures::spawn_local;
use serde::{Deserialize, Serialize};

const TITLE: &str = "egui ex";

#[wasm_bindgen]
pub fn main() {
    eframe::WebLogger::init(log::LevelFilter::Debug).ok();
    let web_options = eframe::WebOptions::default();
    //let app = Box::<MyApp>::default();

    spawn_local(async {
        eframe::WebRunner::new()
            .start(
                TITLE,
                web_options,
                Box::new(|cc| Box::new(MyApp::new(cc))),
            )
            .await
            .expect("failed to start eframe");
    });
}

#[derive(Default)]
#[wasm_bindgen]
pub struct MyApp {
    open_panel: Panel,
    home_panel: HomePanel,
    log_panel: LogPanel,
    config_panel: ConfigPanel,
}

impl MyApp {
    pub fn new(_: &eframe::CreationContext<'_>) -> Self {
        Default::default()
    }
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
                    self.home_panel.ui(ui);
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
    is_recording: bool
}

impl Default for HomePanel {
    fn default() -> Self {
        Self {
            is_recording: false
        }
    }
}

impl HomePanel {
    fn ui(&mut self, ui: &mut Ui) {
        let my_plot = Plot::new("My Plot")
            .legend(Legend::default())
            .height(200.0);

        // let's create a dummy line in the plot
        let graph: Vec<[f64; 2]> = vec![[0.0, 1.0], [2.0, 3.0], [3.0, 2.0]];
        my_plot.show(ui, |plot_ui| {
            plot_ui.line(Line::new(PlotPoints::from(graph)).name("curve"));
        });

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
}

#[wasm_bindgen]
pub struct LogPanel {}

impl Default for LogPanel {
    fn default() -> Self {
        Self {}
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
            //.column(Column::initial(100.0).range(40.0..=300.0))
            //.column(Column::initial(100.0).at_least(40.0).clip(true))
            //.column(Column::remainder())
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
                    ui.strong("Temperature");
                });
                header.col(|ui| {
                    ui.strong("Acceleration");
                });
            })
            .body(|mut body| {
                for row_index in 0..10 {
                    let row_height = 18.0;
                    body.row(row_height, |mut row| {
                        row.col(|ui| {
                            ui.label(row_index.to_string());
                        });
                        row.col(|ui| {
                            ui.add(
                                egui::Label::new("12:00:00.000").wrap(false),
                            );
                        });
                        row.col(|ui| {
                            ui.add(
                                egui::Label::new("27.5").wrap(false),
                            );
                        });
                        row.col(|ui| {
                            ui.add(
                                egui::Label::new("0.2").wrap(false),
                            );
                        });
                    });
                }
            });
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
            temp_enabled: temp_enabled,
            temp_sensitivity: temp_sensitivity,
            accel_enabled: accel_enabled,
            accel_sensitivity: accel_sensitivity,
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

    // Sends config to sever as JSON
    pub async fn send_settings_update(config: Config) {
        let client = reqwest_wasm::Client::new();
        let res = client.post("http://127.0.0.1:8000/update/settings")
            .json(&serde_json::to_string(&config).expect("couldn't serialize"))
            .send()
            .await.expect("no response")
            .text()
            .await;
        debug!("res: {:?}", res);
    }
    
    // Requests config from server, and pack it into a promise
    pub async fn req_settings() -> Option<String> {
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
