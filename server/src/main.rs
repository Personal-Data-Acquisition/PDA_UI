#[macro_use] extern crate rocket;

use rocket::fs::NamedFile;
use rocket::http::{Status, ContentType};
use std::path::{Path, PathBuf};

#[get("/")]
async fn index() -> Result<NamedFile, std::io::Error> {
    NamedFile::open("index.html").await
}

#[get("/<file..>")]
async fn files(file: PathBuf) -> Option<NamedFile> {
    NamedFile::open(Path::new("").join(file)).await.ok()
}

#[post("/update", format = "application/json", data = "<value>")]
async fn update(value: &str) -> Option<&str> {
    println!("{}", value);
    Some(value)
}

#[launch]
fn rocket() -> _ {
    rocket::build()
        .mount("/", routes![index])
        .mount("/", routes![files])
        .mount("/", routes![update])
}
