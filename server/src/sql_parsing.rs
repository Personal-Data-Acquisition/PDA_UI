use sqlx::{sqlite::SqlitePool, Row};

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

        let id = row.get::<i32, usize>(0).to_string();
        array[0] = id;

        let time = row.get::<String, usize>(1).to_string();
        array[1] = time;

        let acc_x = row.get::<f64, usize>(2).to_string();
        array[2] = acc_x;

        let acc_y = row.get::<f64, usize>(3).to_string();
        array[3] = acc_y;

        let acc_z = row.get::<f64, usize>(4).to_string();
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
        
        let fix_type = row.get::<String, usize>(0);
        array[0] = fix_type;

        let fix_time = row.get::<String, usize>(1);
        array[1] = fix_time;

        let fix_date = row.get::<String, usize>(2);
        array[2] = fix_date;

        let latitude = row.get::<f64, usize>(3).to_string();
        array[3] = latitude;

        let longitude = row.get::<f64, usize>(4).to_string();
        array[4] = longitude;

        let altitude = row.get::<f64, usize>(5).to_string();
        array[5] = altitude;

        let speed_over_ground = row.get::<f64, usize>(6).to_string();
        array[6] = speed_over_ground;

        let geoid_separation = row.get::<f64, usize>(7).to_string();
        array[7] = geoid_separation;

        gps.push(array);
    }

    Ok(gps)
}

pub async fn full_temperature() -> Result<Vec<[String; MAX_WIDTH]>, Box<dyn std::error::Error>> {
    let pool = SqlitePool::connect(SQLITE_DATABASE_PATH.as_str()).await?;
    let qry: &str = "SELECT * FROM thermalprobe_data WHERE id IN (SELECT id FROM thermalprobe_data ORDER BY id DESC LIMIT 1000)";
    let temperature = sqlx::query(qry).fetch_all(&pool).await?;

    let mut temp: Vec<[String; MAX_WIDTH]> = Default::default();
    for row in temperature {
        let mut array: [String; MAX_WIDTH] = Default::default();

        let id = row.get::<i32, usize>(0).to_string();
        array[0] = id;

        let timestamp = row.get::<String, usize>(1).to_string();
        array[1] = timestamp;

        let celsius = row.get::<f64, usize>(2).to_string();
        array[2] = celsius;

        temp.push(array);
    }

    Ok(temp)
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

pub async fn latest_data(column: &str, table: &str) -> Result<Vec<[f64; 2]>, Box<dyn std::error::Error>> {
    let pool = SqlitePool::connect(SQLITE_DATABASE_PATH.as_str()).await?;
    let qry = format!("SELECT {} FROM {} WHERE id IN (SELECT id FROM {} ORDER BY id DESC LIMIT 50)", column, table, table);
    let data = sqlx::query(&qry).fetch_all(&pool).await?;

    let mut d: Vec<[f64; 2]> = vec![];
    let mut i: f64 = 0.0;
    for row in data {
        let val: f64 = row.get(0);
        d.push([i, val]);
        i += 1.0;
    }

    Ok(d)
}