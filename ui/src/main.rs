#[macro_use] extern crate rocket;
use rocket::fs::NamedFile;

//The index
#[get("/")]
async fn index() -> Result<NamedFile, std::io::Error> {
    NamedFile::open("index.html").await
}

#[get("/pkg/hello_wasm.js")]
async fn wasm() -> Result<NamedFile, std::io::Error> {
    NamedFile::open("pkg/hello_wasm.js").await
}

#[get("/pkg/hello_wasm_bg.wasm")]
async fn wasm_bg() -> Result<NamedFile, std::io::Error> {
    NamedFile::open("pkg/hello_wasm_bg.wasm").await
}

#[launch]
fn rocket() -> _ {
    rocket::build()
        .mount("/", routes![index])
        .mount("/", routes![wasm])
        .mount("/", routes![wasm_bg])
}
