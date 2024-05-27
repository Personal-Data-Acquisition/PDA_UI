use wasm_bindgen::prelude::*;
use egui::*;
use log::debug;
use crate::utils::PollableValue;
use crate::Config;
use std::sync::Mutex;

const MAX_WIDTH: usize = 12;

#[derive(Debug, PartialEq, Clone, Copy)]
enum DataBases {Acceleration, GPS, Temperature}
static DATA_BASE: Mutex<DataBases> = Mutex::new(DataBases::Acceleration);

struct LogPanelData {
    data: PollableValue<Vec<[String; MAX_WIDTH]>>,   
    time: u16,
}

impl LogPanelData {
    fn new(default: Option<Vec<[String; MAX_WIDTH]>>) -> Self {
        Self {
            data: PollableValue::new(
                default,
                poll_promise::Promise::spawn_local(async move {
                    LogPanel::req_data_full().await // TEMP, TODO genericize this
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
            data: LogPanelData::new(None),
        }
    }
}

impl LogPanel {
    pub fn ui(&mut self, ui: &mut Ui, config: &Config) {
        use egui_extras::{Column, TableBuilder};
        
        let headers: Vec<String>;
        let changed_base: bool;
        
        {
            let mut data_base = DATA_BASE.lock().unwrap();
            let old_base = *data_base;

            egui::ComboBox::from_label("")
                .selected_text(format!("{:?}", *data_base))
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut *data_base, DataBases::Acceleration, "Acceleration");
                    ui.selectable_value(&mut *data_base, DataBases::GPS, "GPS");
                    ui.selectable_value(&mut *data_base, DataBases::Temperature, "Temperature (C)");
                }
            );
            changed_base = old_base != *data_base;
            
            headers = LogPanel::generate_headers(*data_base);
        }
        
        let items = headers.len();

        let table = TableBuilder::new(ui)
            .striped(true)
            .resizable(true)
            .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
            .columns(Column::auto(), headers.len())
            .min_scrolled_height(0.0);

        table
            .header(20.0, |mut header| {
                for label in headers {
                    header.col(|ui| {
                        ui.strong(label);
                    });
                }
            })
            .body(|mut body| {
                let row_height = 18.0;

                if let Some(table_data) = self.data.data.poll() {
                    for entry in table_data {
                        body.row(row_height, |mut row| {
                            row.col(|ui| {
                                ui.label(entry[0].to_string());
                            });
                            for i in 1..items {
                                row.col(|ui| {
                                    ui.add(
                                        egui::Label::new(entry[i].to_string()).wrap(false),
                                    );
                                });
                            }
                        });
                    }
                    // update timer
                    self.data.time += 1;
                    if self.data.time == (config.refresh_time * 60.0) as u16 || changed_base {
                        self.data = LogPanelData::new(
                            self.data.data.value.clone()
                        )
                    }
                }
            });
    }

    /// Requests data of type `Option<Vec<[String; 5]>>` from the server
    async fn req_data_full() -> Option<Vec<[String; MAX_WIDTH]>> {
        let client = reqwest_wasm::Client::new();

        let param: &str;
        {
            let data_base = DATA_BASE.lock().unwrap();
            param = match *data_base {
                DataBases::Acceleration => "acceleration",
                DataBases::GPS => "gps",
                DataBases::Temperature => "temperature",
            };
        }

        let res = match client.get("http://127.0.0.1:8000/req/data/full/".to_owned() + param).send().await {
            Err(why) => {
                debug!("failed to get: {}", why);
                return None;
            },
            Ok(result) => {
                result
            },
        };
        return match res.json::<Vec<[String; MAX_WIDTH]>>().await {
            Err(why) => {
                debug!("failed to parse json: {},", why);
                None
            },
            Ok(result) => {
                Some(result)
            }
        }
    }
    fn generate_headers(choice: DataBases) -> Vec<String> {
        let mut headers: Vec<String> = vec![];

        match choice {
            DataBases::Acceleration => {
                headers.push("Row".to_string());
                headers.push("Time".to_string());
                headers.push("Acceleration X".to_string());
                headers.push("Acceleration Y".to_string());
                headers.push("Acceleration Z".to_string());
            }
            DataBases::GPS => {
                headers.push("Fix Type".to_string());
                headers.push("Fix Time".to_string());
                headers.push("Fix Date".to_string());
                headers.push("Latitude".to_string());
                headers.push("Longitude".to_string());
                headers.push("Altitude".to_string());
                headers.push("Ground Speed".to_string());
                headers.push("Geoid Separation".to_string());
            }
            DataBases::Temperature => {
                headers.push("Row".to_string());
                headers.push("Timestamp".to_string());
                headers.push("Temperature (C)".to_string());
            }
        }

        return headers
    }
}