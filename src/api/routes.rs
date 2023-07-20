use rocket::outcome::Outcome::*;
use rocket::request::{self, FromRequest, Request};
use rocket_contrib::json;
use rocket_contrib::json::*;
use sha2::{Digest, Sha256};

use crate::api::*;
use crate::cache;
use crate::crypto;
use crate::locker;
use crate::persistence;
use crate::user;

// https://github.com/SergioBenitez/Rocket/discussions/2041#discussioncomment-1885738
impl<'a> FromRequest<'a, '_> for Authenticated {
    type Error = &'static str;

    fn from_request(request: &'a Request<'_>) -> request::Outcome<Self, Self::Error> {
        match request.headers().get("AUTHORIZATION").next() {
            Some(val) => match base64::decode(val) {
                Ok(session_id) => match cache::get_bin(&session_id) {
                    Some(session_key) => Success(Authenticated { session_key }),
                    None => {
                        println!("session_id not found in cache");
                        Failure((
                            rocket::http::Status::from_code(401u16)
                                .unwrap_or(rocket::http::Status::InternalServerError),
                            "Unauthorized",
                        ))
                    }
                },
                Err(_) => {
                    println!("Could not base64 decode");
                    Failure((
                        rocket::http::Status::from_code(401u16)
                            .unwrap_or(rocket::http::Status::InternalServerError),
                        "Unauthorized",
                    ))
                }
            },
            None => {
                println!("AUTH header not found");
                Failure((
                    rocket::http::Status::from_code(401u16)
                        .unwrap_or(rocket::http::Status::InternalServerError),
                    "Unauthorized",
                ))
            }
        }
    }
}

//TODO Implement http status code for 500 errors

#[post("/register/start", format = "json", data = "<payload>")]
pub fn register_start(payload: Json<RegisterStart>) -> JsonValue {
    let opaque = crypto::Opaque::new();
    let server_registration_start = opaque.server_side_registration_start(&payload.i, &payload.e);
    let nonce = crypto::create_nonce();
    cache::insert_str(nonce, &payload.c);
    let response_bytes = server_registration_start.message.serialize();
    let response = base64::encode(response_bytes);
    json!({ "id": &nonce, "o": &response })
}

#[post("/register/finish", format = "json", data = "<payload>")]
pub fn register_finish(payload: Json<RegisterFinish>) -> JsonValue {
    match cache::get_str(&payload.id) {
        Some(expected_challenge) => {
            let verifier =
                base64::decode(&payload.v).expect("Could not base64 decode in register_finish!");
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
    let opaque = crypto::Opaque::new();
    let nonce = crypto::create_nonce(); // This is the payload.id to be used throughout entire /login flow and tied to the session_key
    match user::get_user(&payload.e) {
        Ok(user) => {
            let password_file_bytes =
                base64::decode(user.psswd_file).expect("Could not base64 decode in login_start!");
            let server_login_start_result =
                opaque.login_start(&payload.e, &password_file_bytes, &payload.i);
            let server_login_bytes = server_login_start_result.state.serialize().to_vec();
            cache::insert(nonce, server_login_bytes);
            let response_bytes = server_login_start_result.message.serialize();
            let response = base64::encode(response_bytes);
            json!({ "id": &nonce, "o": &response })
        }
        Err(err) => json!({ "id": &nonce, "o": base64::encode(&err) }),
    }
}

#[post("/login/finish", format = "json", data = "<payload>")]
pub fn login_finish(payload: Json<LoginFinish>) -> JsonValue {
    let server_login_bytes = cache::get(&payload.id).unwrap();
    let opaque = crypto::Opaque::new();
    match opaque.login_finish(&server_login_bytes, &payload.i) {
        Ok(session_key) => {
            let rand_bytes = crypto::rand_bytes();
            let ciphertext =
                crypto::encrypt_bytes_with_u32_nonce(&payload.id, &rand_bytes, &session_key);
            let hash = Sha256::digest(&ciphertext).to_vec();
            cache::insert_bin(hash, session_key);
            json!({ "id": &payload.id, "o": base64::encode(rand_bytes) })
        }
        Err(err) => {
            println!("Error during login: {:?}", err);
            json!({ "id": &payload.id, "o": "Failed" })
        }
    }
}

#[post("/login/verify", format = "json", data = "<payload>")]
pub fn login_verify(payload: Json<LoginVerify>) -> JsonValue {
    let client_hash = base64::decode(&payload.i).expect("Could not base64 decode in login_verify!");
    match cache::get_bin(&client_hash) {
        //TODO Need to expire/delete this session_key after x amount of minutes.
        Some(session_key) => {
            //TODO Need to delete the client_hash kv entry since verification is complete.
            let session_key_id = crypto::encrypt_bytes_with_u32_nonce(
                &payload.id,
                &[payload.id.to_be_bytes()].concat(),
                &session_key,
            );
            cache::insert_bin(session_key_id, session_key);
            json!({ "id": 0, "o": "Success" })
        }
        _ => {
            println!("login verification failed: {}", &payload.id);
            json!({ "id": 0, "o": "Failed" })
        }
    }
}

#[post("/locker/register/start", format = "json", data = "<payload>")]
pub fn register_locker_start(
    payload: Json<RegisterLockerStart>,
    _auth: Authenticated,
) -> JsonValue {
    //TODO use _auth.session_key in order to decrypt payload and encrypt response
    let id = &payload.id;
    let _email = &payload.e;
    let input = base64::decode(&payload.i).expect("Could not base64 decode!");
    match locker::register_start(id, &input) {
        Ok(response) => json!({ "id": response.id, "o": response.output }),
        Err(err) => {
            println!("Error in register_locker_start: {:?}", err);
            json!({ "id": err.id, "o": err.msg })
        }
    }
}

#[post("/locker/register/finish", format = "json", data = "<payload>")]
pub fn register_locker_finish(
    payload: Json<RegisterLockerFinish>,
    _auth: Authenticated,
) -> JsonValue {
    //TODO use _auth.session_key in order to decrypt payload and encrypt response
    let id = &payload.id;
    let email = &payload.e;
    let input = base64::decode(&payload.i).expect("Could not base64 decode!");
    let ciphertext = base64::decode(&payload.c).expect("Could not base64 decode!");
    match locker::register_finish(id, email, &input, &ciphertext) {
        Ok(response) => json!({ "id": response.id, "o": response.output }),
        Err(err) => {
            println!("Error in register_locker_finish: {:?}", err);
            json!({ "id": err.id, "o": err.msg })
        }
    }
}

#[post("/locker/open/start", format = "json", data = "<payload>")]
pub fn open_locker_start(payload: Json<OpenLockerStart>, _auth: Authenticated) -> JsonValue {
    //TODO use _auth.session_key in order to decrypt payload and encrypt response
    let locker_id = payload.id.as_str();
    let email = payload.e.as_str();
    let input = base64::decode(&payload.i).expect("Could not base64 decode!");
    match locker::open_start(locker_id, email, &input) {
        Ok(response) => json!({ "id": response.id, "o": response.output, "n": response.nonce }),
        Err(err) => {
            println!("Error in open_locker_start: {:?}", err);
            json!({ "id": err.id, "o": err.msg, "n": err.nonce })
        }
    }
}

#[post("/locker/open/finish", format = "json", data = "<payload>")]
pub fn open_locker_finish(payload: Json<OpenLockerFinish>, _auth: Authenticated) -> JsonValue {
    //TODO use _auth.session_key in order to decrypt payload and encrypt response
    let locker_id = &payload.id;
    let email = &payload.e;
    let input = base64::decode(&payload.i).expect("Could not base64 decode!");
    let nonce = payload.n;
    match locker::open_finish(locker_id, email, &input, nonce) {
        Ok(response) => json!({ "id": response.id, "o": response.output, "n": response.nonce }),
        Err(err) => {
            println!("Error in open_locker_finish: {:?}", err);
            json!({ "id": err.id, "o": err.msg, "n": err.nonce })
        }
    }
}

#[post("/locker/delete/start", format = "json", data = "<payload>")]
pub fn delete_locker_start(payload: Json<DeleteLockerStart>, _auth: Authenticated) -> JsonValue {
    //TODO use _auth.session_key in order to decrypt payload and encrypt response
    let locker_id = payload.id.as_str();
    let email = payload.e.as_str();
    let input = base64::decode(&payload.i).expect("Could not base64 decode!");
    match locker::delete_start(locker_id, email, &input) {
        Ok(response) => json!({ "id": response.id, "o": response.output, "n": response.nonce }),
        Err(err) => {
            println!("Error in delete_locker_start: {:?}", err);
            json!({ "id": err.id, "o": err.msg, "n": err.nonce })
        }
    }
}

#[post("/locker/delete/finish", format = "json", data = "<payload>")]
pub fn delete_locker_finish(payload: Json<DeleteLockerFinish>, _auth: Authenticated) -> JsonValue {
    //TODO use _auth.session_key in order to decrypt payload and encrypt response
    let locker_id = &payload.id;
    let email = &payload.e;
    let input = base64::decode(&payload.i).expect("Could not base64 decode!");
    let nonce = payload.n;
    match locker::delete_finish(locker_id, email, &input, nonce) {
        Ok(response) => json!({ "id": response.id, "o": response.output, "n": response.nonce }),
        Err(err) => {
            println!("Error in delete_locker_finish: {:?}", err);
            json!({ "id": err.id, "o": err.msg, "n": err.nonce })
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
