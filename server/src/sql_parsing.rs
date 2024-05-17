use sqlx::{sqlite::{ SqlitePool, SqliteRow}, Error, Row};

use lazy_static::lazy_static;
use dirs;

lazy_static! {
    static ref SQLITE_DATABASE_PATH: String = {
        // Get the home directory path
        let mut path = dirs::home_dir().expect("Failed to get home directory");
        path.push("sensor_data.db");
        path.to_string_lossy().into_owned()
    };
}

/// Gets acceleration data and packs it into a vector of arrays of id, time, x, y, z
pub async fn full_acceleration() -> Result<Vec<[String; 5]>, Box<dyn std::error::Error>> {
    let pool = SqlitePool::connect(SQLITE_DATABASE_PATH.as_str()).await?;
    let qry: &str = "SELECT id, timestamp, accelerometer_x, accelerometer_y, accelerometer_z FROM accelerometer_data WHERE id IN (SELECT id FROM accelerometer_data ORDER BY id DESC LIMIT 1000)";
    let acceleration = sqlx::query(qry).fetch_all(&pool).await?;

    let mut accel: Vec<[String; 5]> = Default::default();
    for row in acceleration {
        let int: i32 = row.get(0);
        let id = int.to_string();
        let time = row.get(1);
        let mut dec: f32 = row.get(2);
        let acc_x = dec.to_string();
        dec = row.get(3);
        let acc_y = dec.to_string();
        dec = row.get(4);
        let acc_z = dec.to_string();

        accel.push([id, time, acc_x, acc_y, acc_z]);
    }

    Ok(accel)
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