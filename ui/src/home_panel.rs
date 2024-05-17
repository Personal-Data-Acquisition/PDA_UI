use wasm_bindgen::prelude::*;
use std::collections::HashMap;
use egui::*;
use egui_plot::{Legend, Line, Plot, PlotPoints};
use wasm_bindgen_futures::spawn_local;
use walkers::{Tiles, Map, MapMemory, Position, TilesManager, HttpOptions};
use log::debug;
use crate::{send_update, Config, line_drawing::GpsLine, utils::PollableValue};

const GRAPH_COUNT: usize = 4;
const GRAPHS: [Graph; GRAPH_COUNT] = [
    Graph {
        title: "Acceleration X",
        column: "accelerometer_x",
        table: "accelerometer_data",
    },
    Graph {
        title: "Acceleration Y",
        column: "accelerometer_y",
        table: "accelerometer_data",
    },
    Graph {
        title: "Acceleration Z",
        column: "accelerometer_z",
        table: "accelerometer_data",
    },
    Graph {
        title: "Temperature (C)",
        column: "temperature_celsius",
        table: "thermalprobe_data",
    },
];

#[derive(Clone)]
struct Graph {
    title: &'static str,
    column: &'static str,
    table: &'static str,
}

const MAP_HEIGHT: f32 = 600.0;

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
    scroll_offset: f32,
    lowest_edge: f32,
}

/// refers to all of the auto-refreshing data on the home panel
struct HomePanelData {
    time: u16,
    pub data: [PollableValue<Vec<[f64; 2]>>; GRAPH_COUNT],
}

impl HomePanelData {
    fn new(defaults: [Option<Vec<[f64; 2]>>; GRAPH_COUNT]) -> Self {
        Self {
            data: array_init::array_init(|i| {
                PollableValue::new(
                    defaults[i].clone(),
                    poll_promise::Promise::spawn_local(async move {
                        HomePanel::req_data_latest(&GRAPHS[i].column, &GRAPHS[i].table).await   
                    })  
                )
            }),
            time: 0,
        }
    }
}

impl HomePanel {
    pub fn new(ctx: Context) -> Self {
        Self {
            is_recording: false,
            data: HomePanelData::new(array_init::array_init(|_| { None })),
            map_memory: MapMemory::default(),
            providers: providers(ctx),
            gps_points: PollableValue::new(
                Default::default(),
                poll_promise::Promise::spawn_local(async {
                    GpsLine::req_points().await
                })
            ),
            scroll_offset: 0.0,
            lowest_edge: 1800.0,
        }
    }

    pub fn ui(&mut self, ui: &mut Ui, config: &Config) {
        let scroll = ScrollArea::vertical()
        .auto_shrink([false, false])
        .stick_to_bottom(false)
        .show(ui, |ui| {
            let mut ready_count = 0;
            // graphs showing auto-refreshing data
            for i in 0..GRAPH_COUNT {
                if let Some(res) = self.data.data[i].poll() {
                    ready_count += 1;
                    ui.heading(GRAPHS[i].title);
                    let plot = Plot::new(i)
                        .legend(Legend::default())
                        .height(200.0)
                        .allow_scroll(false);
                    let line = Line::new(PlotPoints::from(res)).name(&GRAPHS[i].title);
                    plot.show(ui, |plot_ui| {
                        plot_ui.line(line);
                    });
                }
            }
            // ui.heading(format!("ready count: {ready_count}"));
            // ui.heading(format!("time: {t}", t=self.data.time));
            // if all have been recieved, count up to refresh_time to refresh
            if ready_count == GRAPH_COUNT {
                self.data.time += 1;
                if self.data.time == (config.refresh_time * 60.0) as u16 {
                    self.data = HomePanelData::new(
                        array_init::array_init(|i| {
                            self.data.data[i].poll()
                        })
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
                map = map.with_plugin(GpsLine::new(res, self.scroll_offset));
            }

            let map_corner = ui.cursor().min + Vec2::new( 8.0,MAP_HEIGHT - 40.0);
            ui.add_sized([ui.available_width(), MAP_HEIGHT], map);
            if map_corner[1] <= self.lowest_edge {
                zoom(ui, &mut self.map_memory, map_corner);
            }
        });

        let total_height = scroll.content_size[1] + scroll.inner_rect.min[1];
        self.scroll_offset = ((total_height - scroll.inner_rect.max[1]) - scroll.state.offset.y ) / 2.0;
        self.lowest_edge = scroll.inner_rect.max[1];
    }

    /// Requests data of type `Option<Vec<[f64; 2]>>` from the server
    async fn req_data_latest(column: &str, table: &str) -> Option<Vec<[f64; 2]>> {
        let client = reqwest_wasm::Client::new();
        let url: String = format!("http://127.0.0.1:8000/req/data/latest/{}/{}", column, table);
        let res = match client.get(url).send().await {
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
                debug!("failed to parse json: {}", why);
                None
            },
            Ok(result) => {
                Some(result)
            }
        }
    }
}

pub fn zoom(ui: &Ui, map_memory: &mut MapMemory, location: Pos2) {
    Window::new("Map")
        .collapsible(false)
        .resizable(false)
        .title_bar(false)
        .fixed_pos(location)
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