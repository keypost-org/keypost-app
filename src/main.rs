#![feature(decl_macro)]

#[macro_use]
extern crate rocket;
extern crate rocket_contrib;

use rocket_contrib::serve::StaticFiles;

#[get("/login")]
fn login() -> String {
    "WIP...".to_string()
}

#[get("/logout")]
fn logout() -> String {
    "Goodbye!".to_string()
}

/// https://github.com/SergioBenitez/Rocket/tree/v0.4.10/examples
fn rocket() -> rocket::Rocket {
    rocket::ignite()
        .mount("/", routes![login, logout])
        .mount("/", StaticFiles::from("static").rank(-1))
}

fn main() {
    rocket().launch();
}
