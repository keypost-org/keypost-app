use rocket_contrib::json;
use rocket_contrib::json::{Json, JsonValue};

use crate::api::{LoginFinish, LoginStart, RegisterFinish, RegisterStart};
use crate::cache;
use crate::crypto;
use crate::persistence;

/// curl -X POST -H "Content-Type: application/json" -d '{ "e": "jon@example.com", "i": "Zm9vYmFyCg==", "c": "pkce_challenge" }' http://localhost:8000/register/start
#[post("/register/start", format = "json", data = "<payload>")]
pub fn register_start(payload: Json<RegisterStart>) -> JsonValue {
    println!("{:?}", &payload);
    let opaque = crypto::Opaque::new();
    let server_registration_start = opaque.server_side_registration_start(&payload.i, &payload.e);
    let nonce = rand::random::<u32>();
    cache::insert_str(nonce, &payload.c);
    let response_bytes = server_registration_start.message.serialize();
    let response = base64::encode(response_bytes);
    json!({ "id": &nonce, "o": &response })
}

/// curl -X POST -H "Content-Type: application/json" -d '{ "id": 1234, "e": "jon@example.com", "i": "Zm9vYmFyCg==", "v": "pkce_verifier" }' http://localhost:8000/register/finish
#[post("/register/finish", format = "json", data = "<payload>")]
pub fn register_finish(payload: Json<RegisterFinish>) -> JsonValue {
    println!("{:?}", &payload);

    match cache::get_str(&payload.id) {
        Some(expected_challenge) => {
            let verifier = base64::decode(&payload.v).expect("Could not base64 decode payload.v!");
            let actual_challenge = pkce::code_challenge(&verifier);
            if expected_challenge != actual_challenge {
                return json!({ "id": &payload.id, "o": "bad_nonce_or_code_verifier" });
            }
        }
        None => return json!({ "id": &payload.id, "o": "bad_nonce_or_code_verifier" }),
    }

    let opaque = crypto::Opaque::new();
    let password_file = opaque.server_side_registration_finish(&payload.i);
    match persistence::add_user(&payload.e, base64::encode(password_file).as_str()) {
        Ok(_user) => {
            json!({ "id": &payload.id, "o": "ok" })
        }
        Err(err) => {
            println!("Could not create new user! {}, {:?}", &payload.id, err);
            json!({ "id": &payload.id, "o": "error" })
        }
    }
}

#[post("/login/start", format = "json", data = "<payload>")]
pub fn login_start(payload: Json<LoginStart>) -> JsonValue {
    //TODO Should we do a PKCE-type protocol instead of just a nonce? Maybe OPAQUE internals does this already?
    println!("{:?}", &payload);
    let opaque = crypto::Opaque::new();
    let user = persistence::find_user(&payload.e)
        .expect("No User result!")
        .expect("User not found!");
    let password_file_bytes = base64::decode(user.psswd_file).expect("Could not base64 decode!");
    let server_login_start_result =
        opaque.login_start(&payload.e, &password_file_bytes, &payload.i);
    let nonce = rand::random::<u32>();
    let server_login_bytes = server_login_start_result.state.serialize().to_vec();
    cache::insert(nonce, server_login_bytes);
    let response_bytes = server_login_start_result.message.serialize();
    let response = base64::encode(response_bytes);
    json!({ "id": &nonce, "o": &response })
}

#[post("/login/finish", format = "json", data = "<payload>")]
pub fn login_finish(payload: Json<LoginFinish>) -> JsonValue {
    println!("{:?}", &payload);
    let server_login_bytes = cache::get(&payload.id).unwrap();
    let opaque = crypto::Opaque::new();
    match opaque.login_finish(&server_login_bytes, &payload.i) {
        Ok(()) => json!({ "id": &payload.id, "o": "Success" }),
        Err(err) => {
            println!(
                "Error during login: id={}, {}",
                &payload.id,
                format!("{:?}", err)
            );
            json!({ "id": &payload.id, "o": "Failed" })
        }
    }
}

// To allow (also need a browser extension) CORS during development (-web requests to -app, localhost on different ports)
// TODO Add a build cfg feature around this for local (i.e. non_production) builds only
#[options("/register/start")]
pub fn options_rs() -> JsonValue {
    json!({})
}

#[options("/register/finish")]
pub fn options_rf() -> JsonValue {
    json!({})
}

#[options("/login/start")]
pub fn options_ls() -> JsonValue {
    json!({})
}

#[options("/login/finish")]
pub fn options_lf() -> JsonValue {
    json!({})
}
