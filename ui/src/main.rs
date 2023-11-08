#[macro_use] extern crate rocket;

//The index
#[get("/")]
fn index() -> &'static str {
    "Hello, world!"
}


#[launch]
fn rocket() -> _ {
    rocket::build().mount("/", routes![index])
}
