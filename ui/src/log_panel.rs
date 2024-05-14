use wasm_bindgen::prelude::*;
use egui::*;
use log::debug;
use crate::utils::PollableValue;
use crate::Config;

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
    pub fn ui(&mut self, ui: &mut Ui, config: &Config) {
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