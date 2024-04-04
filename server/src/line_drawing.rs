use std::{fs::File, vec};

const LAT_INDEX: usize = 3;
const LON_INDEX: usize = 4;
const NUM_POINTS: usize = 15;

pub fn last_points() -> Result<Vec<[f64; 2]>, Box<dyn std::error::Error>> {
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