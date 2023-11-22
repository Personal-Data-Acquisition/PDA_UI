use egui_plot::{Legend, Line, Plot, PlotPoints};

const title: &str = "egui ex";

fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        initial_window_size: Some([400.0, 400.0].into()),
        ..Default::default()
    };

    eframe::run_native(
        title,
        options,
        Box::new(|_cc| Box::<MyApp>::default()),
    )
}

#[derive(Default)]
struct MyApp {
    recording: bool,
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let mut plot_rect = None;
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading(title);

            let my_plot = Plot::new("My Plot")
                .legend(Legend::default())
                .height(200.0);

            // let's create a dummy line in the plot
            let graph: Vec<[f64; 2]> = vec![[0.0, 1.0], [2.0, 3.0], [3.0, 2.0]];
            let inner = my_plot.show(ui, |plot_ui| {
                plot_ui.line(Line::new(PlotPoints::from(graph)).name("curve"));
            });

            // Remember the position of the plot
            plot_rect = Some(inner.response.rect);

            ui.horizontal(|ui| {
                if !self.recording {
                    self.recording = ui.button("Record").clicked();
                } else {
                    self.recording = !ui.button("Stop").clicked();
                }
            });
        });
    }
}
