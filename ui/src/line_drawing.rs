use walkers::{Plugin, Projector, Position};
use egui::{Color32, Painter, Response, Stroke};
use log::debug;

const NUM_POINTS: usize = 15;

pub struct GpsLine {
    points: Vec<[f64; 2]>,
    vert_offset: f32,
}

impl GpsLine {
    pub async fn req_points() -> Option<Vec<[f64; 2]>> {
        let client = reqwest_wasm::Client::new();
        let res = match client.get("http://127.0.0.1:8000/req/data/latest/gps_latlon".to_owned()).send().await {
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

    pub fn new(points: Vec<[f64; 2]>, vert_offset: f32) -> Self {
        Self {
            points,
            vert_offset
        }
    }
}

impl Plugin for GpsLine {
    fn draw(&self,
        _response: &Response,
        _gesture_handled: bool,
        painter: Painter,
        projector: &Projector,
    ) {
        let stroke = Stroke::new(3.0, Color32::DARK_RED);

        let mut prev_point = self.points[0];
        for i in 1..NUM_POINTS {
            let current_point = self.points[i];

            let mut point = Position::from_lat_lon(prev_point[0], prev_point[1]);
            let point1 = projector.project(point).to_pos2()
                                - egui::Vec2::new(0.0, self.vert_offset);

            point = Position::from_lat_lon(current_point[0], current_point[1]);
            let point2 = projector.project(point).to_pos2()
                                - egui::Vec2::new(0.0, self.vert_offset);

            painter.line_segment([point1, point2], stroke);
            prev_point = current_point;
        }
    }
}