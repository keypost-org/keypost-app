use rocket_contrib::json;
use rocket_contrib::json::{Json, JsonValue};
use sha2::{Digest, Sha256};

use crate::api::*;
use crate::cache;
use crate::crypto;
use crate::persistence;

//TODO Implement http status code for 500 errors

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
    //TODO Should we do a PKCE-type protocol instead of just a nonce? Maybe OPAQUE internals does this already?
    println!("{:?}", &payload);
    let opaque = crypto::Opaque::new();
    let user = persistence::find_user(&payload.e)
        .expect("No User result!")
        .expect("User not found!");
    let password_file_bytes =
        base64::decode(user.psswd_file).expect("Could not base64 decode in login_start!");
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
        Ok(session_key) => {
            let rand_bytes = crypto::rand_bytes();
            let nonce = [
                payload.id.to_be_bytes(),
                payload.id.to_be_bytes(),
                payload.id.to_be_bytes(),
            ]
            .concat();
            let ciphertext = crypto::encrypt_bytes(&nonce, &rand_bytes, &session_key);
            let hash = Sha256::digest(&ciphertext).to_vec();
            cache::insert_bin(hash, ciphertext);
            json!({ "id": &payload.id, "o": base64::encode(rand_bytes) })
        }
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

#[post("/login/verify", format = "json", data = "<payload>")]
pub fn login_verify(payload: Json<LoginVerify>) -> JsonValue {
    println!("{:?}", &payload);
    let client_hash = base64::decode(&payload.i).expect("Could not base64 decode in login_verify!");
    match cache::get_bin(&client_hash) {
        Some(session_key) => {
            cache::insert(payload.id, session_key);
            json!({ "id": &payload.id, "o": "Success" })
        }
        _ => {
            println!("login verification failed: {}", &payload.id);
            json!({ "id": &payload.id, "o": "Failed" })
        }
    }
}

#[post("/locker/register/start", format = "json", data = "<payload>")]
pub fn register_locker_start(payload: Json<RegisterLockerStart>) -> JsonValue {
    let id = &payload.id;
    let _email = &payload.e;
    let input = base64::decode(&payload.i).expect("Could not base64 decode!");
    let opaque = crypto::Opaque::new();
    match opaque.register_locker_start(id, &input) {
        Ok(output) => json!({ "id": 0, "o": output }),
        Err(err) => {
            println!("Error in register_locker_start: {:?}", err);
            json!({ "id": 0, "o": "There was an error during register_locker_start" })
        }
    }
}

#[post("/locker/register/finish", format = "json", data = "<payload>")]
pub fn register_locker_finish(payload: Json<RegisterLockerFinish>) -> JsonValue {
    let id = &payload.id;
    let email = &payload.e;
    let input = base64::decode(&payload.i).expect("Could not base64 decode!");
    let ciphertext = base64::decode(&payload.c).expect("Could not base64 decode!");
    let opaque = crypto::Opaque::new();
    match opaque.register_locker_finish(id, email, &input, &ciphertext) {
        Ok(()) => json!({ "id": 0, "o": "Success" }),
        Err(err) => {
            println!("Error in register_locker_finish: {:?}", err);
            json!({ "id": 0, "o": "There was an error during register_locker_finish" })
        }
    }
}

#[post("/locker/open/start", format = "json", data = "<payload>")]
pub fn open_locker_start(payload: Json<OpenLockerStart>) -> JsonValue {
    let locker_id = payload.id.as_str();
    let email = payload.e.as_str();
    let input = base64::decode(&payload.i).expect("Could not base64 decode!");
    let (locker_psswd_file, _ciphertext) = persistence::fetch_locker_contents(email, locker_id)
        .expect("Could not get locker_contents");
    let nonce: u32 = crypto::create_nonce();
    let opaque = crypto::Opaque::new();
    match opaque.open_locker_start(locker_id, &input, &locker_psswd_file, nonce) {
        Ok(output) => json!({ "id": 0, "o": output, "n": nonce }),
        Err(err) => {
            println!("Error in open_locker_start: {:?}", err);
            json!({ "id": 0, "o": "There was an error during open_locker_start", "n": nonce })
        }
    }
}

#[post("/locker/open/finish", format = "json", data = "<payload>")]
pub fn open_locker_finish(payload: Json<OpenLockerFinish>) -> JsonValue {
    let locker_id = &payload.id;
    let email = &payload.e;
    let (_locker_psswd_file, ciphertext) = persistence::fetch_locker_contents(email, locker_id)
        .expect("Could not get locker_contents");
    let input = base64::decode(&payload.i).expect("Could not base64 decode!");
    let nonce = payload.n;
    let server_login_bytes: Vec<u8> = cache::get(&nonce)
        .unwrap_or_else(|| panic!("Could not find cached server_login_bytes: {:?}", nonce));
    let opaque = crypto::Opaque::new();
    match opaque.open_locker_finish(&ciphertext, &input, &server_login_bytes) {
        Ok(encrypted_ciphertext) => {
            json!({ "id": 0, "o": base64::encode(encrypted_ciphertext), "n": 0 })
        }
        Err(err) => {
            println!("Error in open_locker_finish: {:?}", err);
            json!({ "id": 0, "o": "There was an error during open_locker_finish", "n": nonce })
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
