use walkers::{
    extras::{Image, Images, Place, Places, Style, Texture},
    Plugin, Projector, Position
};
use egui::{Color32, Painter, Response, Stroke};
use std::{any, fs::File, vec};
use log::debug;

const LAT_INDEX: usize = 3;
const LON_INDEX: usize = 4;
const NUM_POINTS: usize = 15;
pub struct GpsLine {}

impl Plugin for GpsLine {
    fn draw(&self,
        response: &Response,
        _gesture_handled: bool,
        painter: Painter,
        projector: &Projector,
    ) {
        let stroke = Stroke::new(3.0, Color32::DARK_RED);

        let csv_points = last_points().unwrap();

        let mut prev_point = csv_points[0];
        for i in 1..NUM_POINTS {
            let current_point = csv_points[i];

            let mut point = Position::from_lat_lon(prev_point[0], prev_point[1]);
            let point1 = projector.project(point).to_pos2();

            point = Position::from_lat_lon(current_point[0], current_point[1]);
            let point2 = projector.project(point).to_pos2();

            painter.line_segment([point1, point2], stroke);
            prev_point = current_point;
        }
    }
}

// todo: this needs to be run on the server, then served to the client
fn last_points() -> Result<Vec<[f64; 2]>, Box<dyn std::error::Error>> {
    let csv = File::open("gps.csv")?;
    let mut reader = csv::ReaderBuilder::new().has_headers(true).from_reader(csv);
    let mut csv_vec: Vec<[String; 13]> = vec![];

    for result in reader.deserialize() {
        match result {
            Ok(data) => csv_vec.push(data),
            Err(why) => debug!("error in last_points: {}", why),
        };
    }

    let last_entries = csv_vec.iter().rev().take(NUM_POINTS).rev();

    let mut gps_vec: Vec<[f64; 2]> = vec![];
    for result in last_entries {
        let lat: f64 = (&result[LAT_INDEX]).parse().unwrap();
        let lon: f64 = (&result[LON_INDEX]).parse().unwrap();
        gps_vec.push([lat, lon]);
    }

    Ok(gps_vec)
}