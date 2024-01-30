mod utils;

use egui::*;

use egui_plot::{Legend, Line, Plot, PlotPoints};

use wasm_bindgen::prelude::*;
use log::*;

const TITLE: &str = "egui ex";

#[wasm_bindgen]
pub fn main() {
    //let options = eframe::NativeOptions {
    //    initial_window_size: Some([400.0, 400.0].into()),
    //    ..Default::default()
    //};

    eframe::WebLogger::init(log::LevelFilter::Debug).ok();

    let web_options = eframe::WebOptions::default();

    //eframe::run_native(
    //    TITLE,
    //    options,
    //    Box::new(|_cc| Box::<MyApp>::default()),
    //)

    let app = Box::<MyApp>::default();

    wasm_bindgen_futures::spawn_local(async {
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
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
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
                self.is_recording = ui.button("Record").clicked();
            } else {
                self.is_recording = !ui.button("Stop").clicked();
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
                    ui.strong("Temparature");
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
pub struct ConfigPanel {
    temp_enabled: bool,
    temp_sensitivity: f32,
    accel_enabled: bool,
    accel_sensitivity: f32,
}

impl Default for ConfigPanel {
    fn default() -> Self {
        Self {
            temp_enabled: true,
            temp_sensitivity: 8.0,
            accel_enabled: true,
            accel_sensitivity: 8.0,
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
                    ui.checkbox(&mut self.temp_enabled, "Enabled");
                    ui.add_enabled_ui(self.temp_enabled, |ui| {
                        ui.horizontal(|ui| {
                            ui.add(egui::DragValue::
                                   new(&mut self.temp_sensitivity).speed(0.1));
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
                    ui.checkbox(&mut self.accel_enabled, "Enabled");
                    ui.add_enabled_ui(self.accel_enabled, |ui| {
                        ui.horizontal(|ui| {
                            ui.add(egui::DragValue::
                                   new(&mut self.accel_sensitivity).speed(0.1));
                            ui.add(egui::Label::new("Sensitivity"));
                        });
                    });
                });
            });
    }
}