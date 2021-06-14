#![feature(decl_macro)]

#[macro_use] extern crate rocket;
#[macro_use] extern crate serde_derive;

use rocket_contrib::serve::StaticFiles;
use rocket_contrib::json;
use rocket_contrib::json::{Json, JsonValue};

/// https://github.com/SergioBenitez/Rocket/blob/08e5b6dd0dd9d723ca2bd4488ff4a9ef0af8b91b/examples/json/src/main.rs#L22
#[derive(Debug, Deserialize, Serialize)]
struct Message {
    data: String
}

#[get("/login")]
fn login() -> String {
    "WIP...".to_string()
}

#[get("/logout")]
fn logout() -> String {
    "Goodbye!".to_string()
}

/// curl -X POST -H "Content-Type: application/json" -d '{ "data": "bar" }' http://localhost:8000/register
#[post("/register", format = "json", data = "<message>")]
fn register(message: Json<Message>) -> JsonValue {
    println!("{:?}", &message);
    let rand_bytes = (0..32).into_iter().map(|_| rand::random::<u8>()).collect::<Vec<u8>>();
    json!({ "response": String::from_utf8_lossy(&rand_bytes).to_string()})
}

/// https://github.com/SergioBenitez/Rocket/tree/v0.4.10/examples
fn rocket() -> rocket::Rocket {
    rocket::ignite()
        .mount("/", routes![login, logout, register])
        .mount("/", StaticFiles::from("static").rank(-1))
}

fn main() {
    rocket().launch();
}
