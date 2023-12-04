use egui::*;

use egui_plot::{Legend, Line, Plot, PlotPoints};

trait View {
    fn ui(&mut self, ui: &mut egui::Ui);
}

trait Demo {
    fn show(&mut self, ctx: &egui::Context, open: &mut bool);

    fn name(&self) -> &'static str;
}

const TITLE: &str = "egui ex";

fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        initial_window_size: Some([400.0, 400.0].into()),
        ..Default::default()
    };

    eframe::run_native(
        TITLE,
        options,
        Box::new(|_cc| Box::<MyApp>::default()),
    )
}

#[derive(Default)]
struct MyApp {
    open_panel: Panel,
    home_panel: HomePanel,
    log_panel: LogPanel,
    config_panel: ConfigPanel,
}

impl Demo for MyApp {
    fn name(&self) -> &'static str {
        "egui ex"
    }

    fn show(&mut self, ctx: &Context, open: &mut bool) {
        use View as _;
        Window::new(self.name())
            .open(open)
            .default_size(vec2(400.0, 400.0))
            .vscroll(false)
            .show(ctx, |ui| self.ui(ui));
    }
}

impl View for MyApp {
    fn ui(&mut self, ui: &mut Ui) {
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
    }
}

#[derive(PartialEq, Eq)]
enum Panel {
    Home,
    Log,
    Config,
}

impl Default for Panel {
    fn default() -> Self {
        Self::Home
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        //let mut plot_rect = None;
        //egui::CentralPanel::default().show(ctx, |ui| {
        //    ui.heading(TITLE);

        //    let my_plot = Plot::new("My Plot")
        //        .legend(Legend::default())
        //        .height(200.0);

        //    // let's create a dummy line in the plot
        //    let graph: Vec<[f64; 2]> = vec![[0.0, 1.0], [2.0, 3.0], [3.0, 2.0]];
        //    let inner = my_plot.show(ui, |plot_ui| {
        //        plot_ui.line(Line::new(PlotPoints::from(graph)).name("curve"));
        //    });

        //    // Remember the position of the plot
        //    plot_rect = Some(inner.response.rect);

        //    ui.horizontal(|ui| {
        //        if !self.is_recording {
        //            self.is_recording = ui.button("Record").clicked();
        //        } else {
        //            self.is_recording = !ui.button("Stop").clicked();
        //        }
        //    });
        //});

        let mut open = true;

        self.show(ctx, &mut open);
    }
}

// -----------------

struct HomePanel {
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
        ui.horizontal(|ui| {
            ui.label("Home");
        });
        let my_plot = Plot::new("My Plot")
            .legend(Legend::default())
            .height(200.0);

        // let's create a dummy line in the plot
        let graph: Vec<[f64; 2]> = vec![[0.0, 1.0], [2.0, 3.0], [3.0, 2.0]];
        let inner = my_plot.show(ui, |plot_ui| {
            plot_ui.line(Line::new(PlotPoints::from(graph)).name("curve"));
        });

        // Remember the position of the plot
        //plot_rect = Some(inner.response.rect);

        ui.horizontal(|ui| {
            if !self.is_recording {
                self.is_recording = ui.button("Record").clicked();
            } else {
                self.is_recording = !ui.button("Stop").clicked();
            }
        });
    }
}

struct LogPanel {}

impl Default for LogPanel {
    fn default() -> Self {
        Self {}
    }
}

impl LogPanel {
    fn ui(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.label("Log");
        });
    }
}

struct ConfigPanel {}

impl Default for ConfigPanel {
    fn default() -> Self {
        Self {}
    }
}

impl ConfigPanel {
    fn ui(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.label("Config");
        });
    }
}
