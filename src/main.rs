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

mod api;
mod cache;
mod crypto;
mod locker;
mod persistence;
mod user;
mod util;

pub mod models;
pub mod schema;

fn main() -> Result<(), &'static str> {
    init().map_err(|err| panic!("Error pre-initializing app: {:?}", err))?;
    api::init().map_err(|err| panic!("Error initializing api: {:?}", err))
}

fn init() -> Result<(), std::io::Error> {
    crypto::init()?;
    Ok(())
}
