use wasm_bindgen::prelude::*;
use std::collections::HashMap;
use egui::*;
use egui_plot::{Legend, Line, Plot, PlotPoints};
use wasm_bindgen_futures::spawn_local;
use walkers::{Tiles, Map, MapMemory, Position, TilesManager, HttpOptions};
use log::debug;
use crate::{send_update, Config, line_drawing::GpsLine, utils::PollableValue};

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

fn http_options() -> HttpOptions {
    HttpOptions {
        cache: if std::env::var("NO_HTTP_CACHE").is_ok() {
            None
        } else {
            Some(".cache".into())
        },
    }
}

#[wasm_bindgen]
pub struct HomePanel {
    is_recording: bool,
    data: HomePanelData,
    map_memory: MapMemory,
    providers: HashMap<Provider, Box<dyn TilesManager + Send>>,
    gps_points: PollableValue<Vec<[f64; 2]>>,
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

impl HomePanel {
    pub fn new(ctx: Context) -> Self {
        let mut defaults: HashMap<String, std::option::Option<std::vec::Vec<[f64; 2]>>> = HashMap::new();
        for key in HOME_PANEL_KEYS {
            defaults.insert(key.to_string(), None);
        }
        Self {
            is_recording: false,
            data: HomePanelData::new(defaults),
            map_memory: MapMemory::default(),
            providers: providers(ctx),
            gps_points: PollableValue::new(
                Default::default(),
                poll_promise::Promise::spawn_local(async {
                    GpsLine::req_points().await
                })
            ),
        }
    }

    pub fn ui(&mut self, ui: &mut Ui, config: &Config) {
        ScrollArea::vertical()
        .auto_shrink([false, false])
        .stick_to_bottom(false)
        .show(ui, |ui| {
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