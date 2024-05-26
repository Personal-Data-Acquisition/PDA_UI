use sqlx::{sqlite::{ SqlitePool, SqliteRow}, Error, Row};

use lazy_static::lazy_static;
use dirs;

const MAX_WIDTH: usize = 12;

lazy_static! {
    static ref SQLITE_DATABASE_PATH: String = {
        // Get the home directory path
        let mut path = dirs::home_dir().expect("Failed to get home directory");
        path.push("sensor_data.db");
        path.to_string_lossy().into_owned()
    };
}

/// Gets acceleration data and packs it into a vector of arrays of id, time, x, y, z
pub async fn full_acceleration() -> Result<Vec<[String; MAX_WIDTH]>, Box<dyn std::error::Error>> {
    let pool = SqlitePool::connect(SQLITE_DATABASE_PATH.as_str()).await?;
    let qry: &str = "SELECT id, timestamp, accelerometer_x, accelerometer_y, accelerometer_z FROM accelerometer_data WHERE id IN (SELECT id FROM accelerometer_data ORDER BY id DESC LIMIT 1000)";
    let acceleration = sqlx::query(qry).fetch_all(&pool).await?;

    let mut accel: Vec<[String; MAX_WIDTH]> = Default::default();
    for row in acceleration {
        let mut array: [String; MAX_WIDTH] = Default::default();
        let mut int = 0;
        let mut float = 0.0;

        int = row.get(0);
        let id = int.to_string();
        array[0] = id;

        let time = row.get(1);
        array[1] = time;

        float = row.get(2);
        let acc_x = float.to_string();
        array[2] = acc_x;

        float = row.get(3);
        let acc_y = float.to_string();
        array[3] = acc_y;

        float = row.get(4);
        let acc_z = float.to_string();
        array[4] = acc_z;

        accel.push(array);
    }

    Ok(accel)
}

pub async fn full_gps() -> Result<Vec<[String; MAX_WIDTH]>, Box<dyn std::error::Error>> {
    let pool = SqlitePool::connect(SQLITE_DATABASE_PATH.as_str()).await?;
    let qry: &str = "SELECT fix_type, fix_time, fix_date, latitude, longitude, altitude, speed_over_ground, geoid_separation FROM gps_data WHERE fix_time IN (SELECT fix_time FROM gps_data ORDER BY fix_time DESC LIMIT 1000)";
    let acceleration = sqlx::query(qry).fetch_all(&pool).await?;

    let mut gps: Vec<[String; MAX_WIDTH]> = Default::default();
    for row in acceleration {
        let mut array: [String; MAX_WIDTH] = Default::default();
        let mut float = 0.0;
        
        let fix_type = row.get(0);
        array[0] = fix_type;

        let fix_time = row.get(1);
        array[1] = fix_time;

        let fix_date = row.get(2);
        array[2] = fix_date;

        float = row.get(3);
        let latitude = float.to_string();
        array[3] = latitude;

        float = row.get(4);
        let longitude = float.to_string();
        array[4] = longitude;

        float = row.get(5);
        let altitude = float.to_string();
        array[5] = altitude;

        float = row.get(6);
        let speed_over_ground = float.to_string();
        array[6] = speed_over_ground;

        float = row.get(7);
        let geoid_separation = float.to_string();
        array[7] = geoid_separation;

        gps.push(array);
    }

    Ok(gps)
}

pub async fn latest_acceleration_x() -> Result<Vec<[f64; 2]>, Box<dyn std::error::Error>> {
    let pool = SqlitePool::connect(SQLITE_DATABASE_PATH.as_str()).await?;
    let qry: &str = "SELECT accelerometer_x FROM accelerometer_data WHERE id IN (SELECT id FROM accelerometer_data ORDER BY id DESC LIMIT 50)";
    let acceleration_x = sqlx::query(qry).fetch_all(&pool).await?;

    let mut accel: Vec<[f64; 2]> = vec![];
    let mut i: f64 = 0.0;
    for row in data {
        let val: f64 = row.get(0);
        d.push([i, val]);
        i += 1.0;
    }

    Ok(accel)
}

pub async fn latest_acceleration_y() -> Result<Vec<[f64; 2]>, Box<dyn std::error::Error>> {
    let pool = SqlitePool::connect(SQLITE_DATABASE_PATH.as_str()).await?;
    let qry: &str = "SELECT accelerometer_y FROM accelerometer_data WHERE id IN (SELECT id FROM accelerometer_data ORDER BY id DESC LIMIT 50)";
    let acceleration_y = sqlx::query(qry).fetch_all(&pool).await?;

    let mut accel: Vec<[f64; 2]> = vec![];
    let mut i: f64 = 0.0;
    for row in acceleration_y {
        let val: f64 = row.get(0);
        accel.push([i, val]);
        i += 1.0;
    }

    Ok(accel)
}

pub async fn latest_acceleration_z() -> Result<Vec<[f64; 2]>, Box<dyn std::error::Error>> {
    let pool = SqlitePool::connect(SQLITE_DATABASE_PATH.as_str()).await?;
    let qry: &str = "SELECT accelerometer_z FROM accelerometer_data WHERE id IN (SELECT id FROM accelerometer_data ORDER BY id DESC LIMIT 50)";
    let acceleration_z = sqlx::query(qry).fetch_all(&pool).await?;

    let mut accel: Vec<[f64; 2]> = vec![];
    let mut i: f64 = 0.0;
    for row in acceleration_z {
        let val: f64 = row.get(0);
        accel.push([i, val]);
        i += 1.0;
    }

    Ok(accel)
}

pub async fn latest_gps_latlon() -> Result<Vec<[f64; 2]>, Box<dyn std::error::Error>> {
    let pool = SqlitePool::connect(SQLITE_DATABASE_PATH.as_str()).await?;
    let qry: &str = "SELECT latitude, longitude FROM gps_data WHERE fix_time IN (SELECT fix_time FROM gps_data ORDER BY fix_time DESC LIMIT 50) AND (fix_type != 'Invalid')";
    let gps_data = sqlx::query(qry).fetch_all(&pool).await?;
    
    let mut gps: Vec<[f64; 2]> = vec![];
    for row in gps_data {
        let lat: f64 = row.get(0);
        let lon: f64 = row.get(1);
        gps.push([lat, lon]);
    }

    Ok(gps)
}