use walkers::{
    extras::{Image, Images, Place, Places, Style, Texture},
    Plugin, Projector, Position
};
use egui::{Color32, Painter, Response, Stroke};

pub struct GpsLine {}

impl Plugin for GpsLine {
    fn draw(&self,
        response: &Response,
        _gesture_handled: bool,
        painter: Painter,
        projector: &Projector,
    ) {
        let point1 = Position::from_lat_lon(44.56295442928084, -123.27819700306057);
        let point1 = projector.project(point1).to_pos2();

        let point2 = Position::from_lat_lon(44.564873061658496, -123.27970976877855);
        let point2 = projector.project(point2).to_pos2();

        let points = [point1, point2];
        let stroke = Stroke::new(3.0, Color32::DARK_RED);

        painter.line_segment(points, stroke);
    }
}