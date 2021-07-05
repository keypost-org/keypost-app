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
    u: String,
    i: String,
}

#[derive(Debug, Deserialize, Serialize)]
struct RegisterFinish {
    id: u32,
    u: String,
    i: String,
}

#[derive(Debug, Deserialize, Serialize)]
struct LoginStart {
    u: String,
    i: String,
}

#[derive(Debug, Deserialize, Serialize)]
struct LoginFinish {
    id: u32,
    u: String,
    i: String,
}

#[get("/login")]
fn login() -> String {
    "WIP...".to_string()
}

#[get("/logout")]
fn logout() -> String {
    "Goodbye!".to_string()
}

/// curl -X POST -H "Content-Type: application/json" -d '{ "u": "jon", "i": "Zm9vYmFyCg==" }' http://localhost:8000/register/start
#[post("/register/start", format = "json", data = "<payload>")]
fn register_start(payload: Json<RegisterStart>) -> JsonValue {
    println!("{:?}", &payload);
    let opaque = crypto::Opaque::new();
    let server_registration_start = opaque.server_side_registration_start(&payload.i);
    let nonce = rand::random::<u32>();
    let server_registration_bytes = server_registration_start.state.serialize();
    cache::insert(nonce, server_registration_bytes);
    let response_bytes = server_registration_start.message.serialize();
    let response = base64::encode(response_bytes);
    json!({ "id": &nonce, "o": &response })
}

/// curl -X POST -H "Content-Type: application/json" -d '{ "id": 1234, "u": "jon", "i": "Zm9vYmFyCg==" }' http://localhost:8000/register/finish
#[post("/register/finish", format = "json", data = "<payload>")]
fn register_finish(payload: Json<RegisterFinish>) -> JsonValue {
    println!("{:?}", &payload);
    let server_registration_bytes = cache::get(&payload.id).expect("Could not find in cache!");
    let opaque = crypto::Opaque::new();
    let password_file =
        opaque.server_side_registration_finish(&payload.i, &server_registration_bytes);
    cache::add_user(payload.u.clone(), password_file);
    json!({ "id": &payload.id, "o": "ok" })
}

#[post("/login/start", format = "json", data = "<payload>")]
fn login_start(payload: Json<LoginStart>) -> JsonValue {
    println!("{:?}", &payload);
    let opaque = crypto::Opaque::new();
    let password_file_bytes = cache::get_user(&payload.u).expect("Could not find in cache!");
    let server_login_start_result = opaque.login_start(&password_file_bytes, &payload.i);
    let nonce = rand::random::<u32>();
    let server_login_bytes = server_login_start_result.state.serialize();
    cache::insert(nonce, server_login_bytes);
    let response_bytes = server_login_start_result.message.serialize();
    let response = base64::encode(response_bytes);
    json!({ "id": &nonce, "o": &response })
}

#[post("/login/finish", format = "json", data = "<payload>")]
fn login_finish(payload: Json<LoginFinish>) -> JsonValue {
    println!("{:?}", &payload);
    let server_login_bytes = cache::get(&payload.id).unwrap();
    let opaque = crypto::Opaque::new();
    let response_bytes = opaque.login_finish(&server_login_bytes, &payload.i);
    let response = base64::encode(response_bytes);
    json!({ "id": &payload.id, "o": &response })
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
