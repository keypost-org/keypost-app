#![feature(decl_macro)]

#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate rocket;
#[macro_use]
extern crate serde_derive;

use rocket_contrib::json;
use rocket_contrib::json::{Json, JsonValue};
use rocket_contrib::serve::StaticFiles;

use std::collections::HashMap;
use std::sync::Mutex;

mod crypto;

lazy_static! {
    static ref CACHE: Mutex<HashMap<u32, Vec<u8>>> = {
        let map = HashMap::new();
        Mutex::new(map)
    };
}

/// https://github.com/SergioBenitez/Rocket/blob/08e5b6dd0dd9d723ca2bd4488ff4a9ef0af8b91b/examples/json/src/main.rs#L22
#[derive(Debug, Deserialize, Serialize)]
struct Message {
    id: u32,
    data: String,
}

#[get("/login")]
fn login() -> String {
    "WIP...".to_string()
}

#[get("/logout")]
fn logout() -> String {
    "Goodbye!".to_string()
}

/// curl -X POST -H "Content-Type: application/json" -d '{ "id": 0, "data": "bar" }' http://localhost:8000/register/start
#[post("/register/start", format = "json", data = "<message>")]
fn register_start(message: Json<Message>) -> JsonValue {
    println!("{:?}", &message);
    let opaque = crypto::Opaque::new();
    let server_registration_start = opaque.server_side_registration_start(&message.data);

    let nonce = rand::random::<u32>();
    let server_registration_bytes = server_registration_start.state.serialize();
    let mut cache = CACHE.lock().unwrap();
    cache.insert(nonce, server_registration_bytes);

    let response_bytes = server_registration_start.message.serialize();
    let response = base64::encode(response_bytes);
    json!({ "id": &nonce, "response": &response })
}

/// curl -X POST -H "Content-Type: application/json" -d '{ "id": 1234, "data": "bar" }' http://localhost:8000/register/file
#[post("/register/finish", format = "json", data = "<message>")]
fn register_finish(message: Json<Message>) -> JsonValue {
    println!("{:?}", &message);
    let cache = CACHE.lock().unwrap();
    let server_registration_bytes = cache.get(&message.id).unwrap();
    let opaque = crypto::Opaque::new();
    let password_file =
        opaque.server_side_registration_finish(&message.data, server_registration_bytes);
    let response = base64::encode(password_file);
    json!({ "id": &message.id, "response": &response })
}

/// https://github.com/SergioBenitez/Rocket/tree/v0.4.10/examples
fn rocket() -> rocket::Rocket {
    rocket::ignite()
        .mount("/", routes![login, logout, register_start, register_finish])
        .mount("/", StaticFiles::from("static").rank(-1))
}

fn main() {
    rocket().launch();
}
