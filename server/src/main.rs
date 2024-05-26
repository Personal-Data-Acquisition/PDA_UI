#[macro_use] extern crate rocket;

mod sql_parsing;

use rocket::fs::NamedFile;
use std::path::{Path, PathBuf};
use std::fs;
use std::io::Write;

#[get("/")]
async fn index() -> Result<NamedFile, std::io::Error> {
    NamedFile::open("index.html").await
}

#[get("/<file..>")]
async fn files(file: PathBuf) -> Option<NamedFile> {
    NamedFile::open(Path::new("").join(file)).await.ok()
}

#[get("/req/settings")]
async fn req_settings() -> Result<String, std::io::Error> {
    fs::read_to_string("settings.json")
}

#[get("/req/data/latest/<param>")]
async fn req_data_latest(param: &str) -> Result<String, String> {
    let content = match param {
        "acceleration_x" => sql_parsing::latest_acceleration_x().await,
        "acceleration_y" => sql_parsing::latest_acceleration_y().await,
        "acceleration_z" => sql_parsing::latest_acceleration_z().await,
        "gps_latlon" => sql_parsing::latest_gps_latlon().await,
        // todo: more data types
        &_ => Err("invalid data type for req_data_latest".into()),
    };
    match content {
        Ok(c) => match serde_json::to_string(&c) {
            Ok(s) => Ok(s),
            Err(why) => Err(format!("could not deserialize: {}", why)),
        },
        Err(why) => Err(format!("invalid content: {}", why)),
    }
}

#[get("/req/data/full/<param>")]
async fn req_data_full(param: &str) -> Result<String, String> {
    let content = match param {
        "acceleration" => sql_parsing::full_acceleration().await,
        "gps" => sql_parsing::full_gps().await,
        // todo: more data types
        &_ => Err("invalid data type for req_data_full".into()),
    };
    match content {
        Ok(c) => match serde_json::to_string(&c) {
            Ok(s) => Ok(s),
            Err(why) => Err(format!("could not deserialize: {}", why)),
        },
        Err(why) => Err(format!("invalid content: {}", why)),
    }
}

#[post("/update", format = "application/json", data = "<value>")]
async fn update(value: &str) -> Option<&str> {
    println!("{}", value);
    Some(value)
}

#[post("/update/settings", format = "application/json", data = "<value>")]
async fn update_settings(value: &str) -> Result<&str, String> {
    println!("{}", value);
    let mut file = match fs::File::create("settings.json") {
        Err(why) => return Err(format!("couldn't create settings: {why}")),
        Ok(file) => file,
    };
    if let Err(why) = file.write_all(value.as_bytes()) {
        return Err(format!("couldn't write to settings: {why}"))
    }
    Ok(value)
}

#[launch]
fn rocket() -> _ {
    rocket::build()
        .configure(rocket::Config::figment().merge(("port", 8000)))
        .mount("/", routes![index])
        .mount("/", routes![files])
        .mount("/", routes![update])
        .mount("/", routes![update_settings])
        .mount("/", routes![req_settings])
        .mount("/", routes![req_data_latest])
        .mount("/", routes![req_data_full])
}
