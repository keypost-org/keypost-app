#![feature(decl_macro)]

#[macro_use]
extern crate diesel;
extern crate dotenv;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate rocket;
#[macro_use]
extern crate serde_derive;

use rocket_contrib::serve::StaticFiles;

mod api;
mod cache;
mod crypto;
mod persistence;
mod util;

pub mod models;
pub mod schema;

/// https://github.com/SergioBenitez/Rocket/tree/v0.4.10/examples
fn rocket() -> rocket::Rocket {
    persistence::get_active_users().expect("Could not get users!");

    rocket::ignite()
        .mount(
            "/",
            routes![
                api::register_start,
                api::register_finish,
                api::login_start,
                api::login_finish,
                api::options
            ],
        )
        .mount("/", StaticFiles::from("static").rank(-1))
}

fn main() {
    init().expect("Could not initialize app before launching rocket!");
    rocket().launch();
}

fn init() -> Result<(), std::io::Error> {
    crypto::init()?;
    Ok(())
}
