#[macro_use] extern crate rocket;

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
async fn req_settings() -> String {
    let content = fs::read_to_string("settings.json")
        .expect("Couldn't read settings");
    print!("{}", content);
    content
}

#[post("/update", format = "application/json", data = "<value>")]
async fn update(value: &str) -> Option<&str> {
    println!("{}", value);
    Some(value)
}

#[post("/update/settings", format = "application/json", data = "<value>")]
async fn update_settings(value: &str) -> Option<&str> {
    println!("{}", value);
    let mut file = match fs::File::create("settings.json") {
        Err(why) => panic!("couldn't create settings: {}", why),
        Ok(file) => file,
    };
    match file.write_all(value.as_bytes()) {
        Err(why) => panic!("couldn't write to settings: {}", why),
        Ok(_) => println!("successfully wrote to settings"),
    };
    Some(value)
}

#[launch]
fn rocket() -> _ {
    rocket::build()
        .mount("/", routes![index])
        .mount("/", routes![files])
        .mount("/", routes![update])
        .mount("/", routes![update_settings])
        .mount("/", routes![req_settings])
}
