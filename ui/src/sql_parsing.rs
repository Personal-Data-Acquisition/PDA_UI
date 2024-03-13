use sqlx::{sqlite::{SqliteConnectOptions, SqlitePool, SqliteRow}, Error, Row};
use std::{future::Future, path::Path};

const DB_NAME: &str = "sensor_data.db";
const DB_URL: &str = "sqlite://sensor_data.db";

// Function taken from: https://stackoverflow.com/questions/72763578/how-to-create-a-sqlite-database-with-rust-sqlx
async fn connect_file(filename: impl AsRef<Path>) -> impl Future<Output = Result<SqlitePool, Error>> {
    let options = SqliteConnectOptions::new()
        .filename(filename)
        .create_if_missing(false);

    SqlitePool::connect_with(options)
}

pub async fn pull_acceleration() -> Result<Vec<SqliteRow>, Box<dyn std::error::Error>> {
    let pool = SqlitePool::connect(DB_URL).await?;
    let acceleration = sqlx::query("SELECT * FROM accelerometer_data").fetch_all(&pool).await?;

    Ok(acceleration)
}

pub async fn pull_acceleration_x() -> Result<Vec<[f64; 2]>, Box<dyn std::error::Error>> {
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