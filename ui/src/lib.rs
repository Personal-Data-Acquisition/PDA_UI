mod utils;
mod line_drawing;
mod home_panel;
mod log_panel;
mod config_panel;

use wasm_bindgen::prelude::*;
use std::collections::HashMap;
use log::debug;
use wasm_bindgen_futures::spawn_local;
use egui::Context;
use crate::{home_panel::*, log_panel::*, config_panel::*};

const TITLE: &str = "Personal Data Acquisition";

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
}

impl MyApp {
    fn new(ctx: Context) -> Self {
        Self {
            open_panel: Panel::default(),
            home_panel: HomePanel::new(ctx),
            log_panel: LogPanel::default(),
            config_panel: ConfigPanel::default(),
        }
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
                    self.home_panel.ui(ui, &self.config_panel.config);
                },
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
    let res = match client.post(url).json(body).send().await {
            Ok(r) => r.text().await,
            Err(e) => Err(e)
        };
    debug!("res: {:?}", res);
}