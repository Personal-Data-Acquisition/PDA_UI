use sqlx::{sqlite::{SqliteConnectOptions, SqlitePool, SqliteRow}, Error};
use std::{future::Future, path::Path, env};
use futures::executor;

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