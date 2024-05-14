use wasm_bindgen::prelude::*;
use egui::*;
use wasm_bindgen_futures::spawn_local;
use log::debug;
use serde::{Deserialize, Serialize};

#[wasm_bindgen]
#[derive(Copy, Clone, Deserialize, Serialize)]
pub struct Config {
    pub temp_enabled: bool,
    pub temp_sensitivity: f32,
    pub accel_enabled: bool,
    pub accel_sensitivity: f32,
    pub refresh_time: f32, // seconds
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
    pub config: Config,
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
    pub fn ui(&mut self, ui: &mut Ui) {
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
        let res = match &serde_json::to_string(&config) {
            Ok(j) => {
                match client.post("http://127.0.0.1:8000/update/settings").json(j).send().await {
                    Ok(r) => r.text().await,
                    Err(e) => Err(e),
                }
            },
            Err(e) => {
                debug!("couldn't serialize: {:?}", e);
                return;
            },
        };
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