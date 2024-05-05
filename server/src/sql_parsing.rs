use sqlx::{sqlite::{ SqlitePool, SqliteRow}, Error, Row};

const DB_URL: &str = "sqlite://sensor_data.db";

/// Gets acceleration data and packs it into a vector of arrays of id, time, x, y, z
pub async fn full_acceleration() -> Result<Vec<[String; 5]>, Box<dyn std::error::Error>> {
    let pool = SqlitePool::connect(DB_URL).await?;
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

pub async fn latest_acceleration_x() -> Result<Vec<[f64; 2]>, Box<dyn std::error::Error>> {
    let pool = SqlitePool::connect(DB_URL).await?;
    let qry: &str = "SELECT accelerometer_x FROM accelerometer_data WHERE id IN (SELECT id FROM accelerometer_data ORDER BY id DESC LIMIT 50)";
    let acceleration_x = sqlx::query(qry).fetch_all(&pool).await?;

    let mut accel: Vec<[f64; 2]> = vec![];
    let mut i: f64 = 0.0;
    for row in acceleration_x {
        let val: f64 = row.get(0);
        accel.push([i, val]);
        i += 1.0;
    }

    Ok(accel)
}

pub async fn latest_acceleration_y() -> Result<Vec<[f64; 2]>, Box<dyn std::error::Error>> {
    let pool = SqlitePool::connect(DB_URL).await?;
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
    let pool = SqlitePool::connect(DB_URL).await?;
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