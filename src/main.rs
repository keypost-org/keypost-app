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

mod cache;
mod crypto;

/// https://github.com/SergioBenitez/Rocket/blob/08e5b6dd0dd9d723ca2bd4488ff4a9ef0af8b91b/examples/json/src/main.rs#L22
#[derive(Debug, Deserialize, Serialize)]
struct RegisterStart {
    data: String,
}

#[derive(Debug, Deserialize, Serialize)]
struct RegisterFinish {
    id: u32,
    data: String,
}

#[derive(Debug, Deserialize, Serialize)]
struct LoginStart {
    file: String,
    data: String,
}

#[derive(Debug, Deserialize, Serialize)]
struct LoginFinish {
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

/// curl -X POST -H "Content-Type: application/json" -d '{ "data": "bar" }' http://localhost:8000/register/start
#[post("/register/start", format = "json", data = "<payload>")]
fn register_start(payload: Json<RegisterStart>) -> JsonValue {
    println!("{:?}", &payload);
    let opaque = crypto::Opaque::new();
    let server_registration_start = opaque.server_side_registration_start(&payload.data);
    let nonce = rand::random::<u32>();
    let server_registration_bytes = server_registration_start.state.serialize();
    cache::insert(nonce, server_registration_bytes);
    let response_bytes = server_registration_start.message.serialize();
    let response = base64::encode(response_bytes);
    json!({ "id": &nonce, "data": &response })
}

/// curl -X POST -H "Content-Type: application/json" -d '{ "id": 1234, "data": "bar" }' http://localhost:8000/register/file
#[post("/register/finish", format = "json", data = "<payload>")]
fn register_finish(payload: Json<RegisterFinish>) -> JsonValue {
    println!("{:?}", &payload);
    let server_registration_bytes = cache::get(&payload.id).unwrap();
    let opaque = crypto::Opaque::new();
    let password_file =
        opaque.server_side_registration_finish(&payload.data, &server_registration_bytes);
    let response = base64::encode(password_file);
    json!({ "id": &payload.id, "data": &response })
}

#[post("/login/start", format = "json", data = "<payload>")]
fn login_start(payload: Json<LoginStart>) -> JsonValue {
    println!("{:?}", &payload);
    let opaque = crypto::Opaque::new();
    let server_login_start_result = opaque.login_start(&payload.data, &payload.file);
    let nonce = rand::random::<u32>();
    let server_login_bytes = server_login_start_result.state.serialize();
    cache::insert(nonce, server_login_bytes);
    let response_bytes = server_login_start_result.message.serialize();
    let response = base64::encode(response_bytes);
    json!({ "id": &nonce, "data": &response })
}

#[post("/login/finish", format = "json", data = "<payload>")]
fn login_finish(payload: Json<LoginFinish>) -> JsonValue {
    println!("{:?}", &payload);
    let server_login_bytes = cache::get(&payload.id).unwrap();
    let opaque = crypto::Opaque::new();
    let response_bytes = opaque.login_finish(&server_login_bytes, &payload.data);
    let response = base64::encode(response_bytes);
    json!({ "id": &payload.id, "data": &response })
}

/// https://github.com/SergioBenitez/Rocket/tree/v0.4.10/examples
fn rocket() -> rocket::Rocket {
    rocket::ignite()
        .mount(
            "/",
            routes![
                login,
                logout,
                register_start,
                register_finish,
                login_start,
                login_finish
            ],
        )
        .mount("/", StaticFiles::from("static").rank(-1))
}

fn main() {
    rocket().launch();
}
