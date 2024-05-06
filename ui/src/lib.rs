mod utils;
mod line_drawing;
mod home_panel;
mod log_panel;
mod config_panel;

use egui::*;
use wasm_bindgen::prelude::*;
use std::collections::HashMap;
use log::debug;
use wasm_bindgen_futures::spawn_local;
use walkers::{Tiles, Map, MapMemory, Position, TilesManager, HttpOptions};
use crate::{
    line_drawing::GpsLine, 
    utils::PollableValue, 
    home_panel::*, 
    log_panel::*, 
    config_panel::*};

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
    let res = match client.post(url).json(body).send().await {
            Ok(r) => r.text().await,
            Err(e) => Err(e)
        };
    debug!("res: {:?}", res);
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