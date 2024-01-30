use diesel::result::DatabaseErrorKind;
use rocket::http::Status;
use rocket::outcome::Outcome::*;
use rocket::request::{self, FromRequest, Request};
use rocket_contrib::json;
use rocket_contrib::json::{Json, JsonValue};
use sha2::{Digest, Sha256};

use crate::api::*;
use crate::cache;
use crate::crypto;
use crate::locker;
use crate::persistence;
use crate::user;

// https://github.com/SergioBenitez/Rocket/discussions/2041#discussioncomment-1885738
impl<'a> FromRequest<'a, '_> for Authenticated {
    type Error = ApiError;

    fn from_request(request: &'a Request<'_>) -> request::Outcome<Self, Self::Error> {
        match request.headers().get("AUTHORIZATION").next() {
            Some(val) => match base64::decode(val) {
                Ok(session_id) => match cache::get_bin(&session_id) {
                    Some(session_key) => Success(Authenticated { session_key }),
                    None => {
                        println!("session_id not found in cache");
                        Failure((Status::Unauthorized, ApiError::NotAuthenticated))
                    }
                },
                Err(_) => {
                    println!("Could not base64 decode");
                    Failure((Status::Unauthorized, ApiError::NotAuthenticated))
                }
            },
            None => {
                println!("AUTH header not found");
                Failure((Status::Unauthorized, ApiError::NotAuthenticated))
            }
        }
    }
}

#[post("/register/start", format = "json", data = "<payload>")]
pub fn register_start(payload: Json<RegisterStart>) -> Result<JsonValue, ApiError> {
    let server_registration_start = crypto::server_side_registration_start(&payload.i, &payload.e)?;
    let nonce = crypto::create_nonce();
    cache::insert_str(nonce, &payload.c);
    let response_bytes = server_registration_start.message.serialize();
    let response = base64::encode(response_bytes);
    Ok(json!({ "id": &nonce, "o": &response }))
}

#[post("/register/finish", format = "json", data = "<payload>")]
pub fn register_finish(payload: Json<RegisterFinish>) -> Result<JsonValue, ApiError> {
    match cache::get_str(&payload.id) {
        Some(expected_challenge) => {
            let verifier = base64::decode(&payload.v).map_err(ApiError::BadRequestDecode)?;
            let actual_challenge = pkce::code_challenge(&verifier);
            if expected_challenge != actual_challenge {
                return Ok(json!({ "id": &payload.id, "o": "bad_nonce_or_code_verifier" }));
            }
        }
        None => return Err(ApiError::BadRequest),
    }

    let password_file = crypto::server_side_registration_finish(&payload.i);
    match persistence::add_user(&payload.e, base64::encode(password_file).as_str()) {
        Ok(_user) => Ok(json!({ "id": &payload.id, "o": "ok" })),
        Err(diesel::result::Error::DatabaseError(DatabaseErrorKind::UniqueViolation, info)) => {
            println!("UniqueViolation: {:?}", info);
            Err(ApiError::BadConfirmationKeyOrWrongEmail)
        }
        Err(err) => {
            println!("Could not create new user! {}, {:?}", &payload.id, err);
            Err(ApiError::UnknownError)
        }
    }
}

#[post("/login/start", format = "json", data = "<payload>")]
pub fn login_start(payload: Json<LoginStart>) -> Result<JsonValue, ApiError> {
    let nonce = crypto::create_nonce(); // This is the payload.id to be used throughout entire /login flow and tied to the session_key
    match user::get_user(&payload.e) {
        Ok(user) => {
            let password_file_bytes =
                base64::decode(user.psswd_file).expect("Could not base64 decode in login_start!");
            let server_login_start_result =
                crypto::login_start(&payload.e, &password_file_bytes, &payload.i);
            let server_login_bytes = server_login_start_result.state.serialize().to_vec();
            cache::insert(nonce, server_login_bytes);
            let response_bytes = server_login_start_result.message.serialize();
            let response = base64::encode(response_bytes);
            Ok(json!({ "id": &nonce, "o": &response }))
        }
        Err(msg) => Err(ApiError::LoginError(msg)), //json!({ "id": &nonce, "o": base64::encode(err) }),
    }
}

#[post("/login/finish", format = "json", data = "<payload>")]
pub fn login_finish(payload: Json<LoginFinish>) -> Result<JsonValue, ApiError> {
    let server_login_bytes = cache::get(&payload.id).unwrap();
    match crypto::login_finish(&server_login_bytes, &payload.i) {
        Ok(session_key) => {
            let rand_bytes = crypto::rand_bytes();
            let ciphertext =
                crypto::encrypt_bytes_with_u32_nonce(&payload.id, &session_key, &rand_bytes);
            let client_hash = Sha256::digest(&ciphertext).to_vec();
            //TODO Need to expire this client_hash/session_key incase /login/verify never completes (i.e. failed login attempts will pile up!).
            cache::insert_bin(client_hash, session_key);
            Ok(json!({ "id": &payload.id, "o": base64::encode(rand_bytes) }))
        }
        Err(err) => {
            println!("Error during login: {:?}", err);
            Err(ApiError::BadRequestProtocol)
            //json!({ "id": &payload.id, "o": "Failed" })
        }
    }
}

#[post("/login/verify", format = "json", data = "<payload>")]
pub fn login_verify(payload: Json<LoginVerify>) -> Result<JsonValue, ApiError> {
    let client_hash = base64::decode(&payload.i).map_err(ApiError::BadRequestDecode)?;
    match cache::get_bin(&client_hash) {
        Some(session_key) => {
            let session_key_id = crypto::encrypt_bytes_with_u32_nonce(
                &payload.id,
                &session_key,
                &[payload.id.to_be_bytes()].concat(),
            );
            //TODO Need to expire/delete this new session_key entry after x amount of minutes (i.e. user's session expired).
            cache::insert_bin(session_key_id, session_key);
            cache::delete_bin(&client_hash); // i.e. login verification complete!
            Ok(json!({ "id": 0, "o": "Success" }))
        }
        _ => {
            println!("login verification failed: {}", &payload.id);
            Err(ApiError::LoginError("Failed".to_string()))
            // json!({ "id": 0, "o": "Failed" })
        }
    }
}

#[post("/locker/register/start", format = "json", data = "<payload>")]
pub fn register_locker_start(
    payload: Json<RegisterLockerStart>,
    _auth: Authenticated,
) -> Result<JsonValue, ApiError> {
    //TODO use _auth.session_key in order to decrypt payload and encrypt response
    let id = &payload.id;
    let _email = &payload.e;
    let input = base64::decode(&payload.i).expect("Could not base64 decode!");
    match locker::register_start(id, &input) {
        Ok(response) => Ok(json!({ "id": response.id, "o": response.output })),
        Err(err) => {
            println!("Error in register_locker_start: {:?}", err);
            Err(err)
            // json!({ "id": err.id, "o": err.msg })
        }
    }
}

#[post("/locker/register/finish", format = "json", data = "<payload>")]
pub fn register_locker_finish(
    payload: Json<RegisterLockerFinish>,
    _auth: Authenticated,
) -> Result<JsonValue, ApiError> {
    //TODO use _auth.session_key in order to decrypt payload and encrypt response
    let id = &payload.id;
    let email = &payload.e;
    let input = base64::decode(&payload.i).expect("Could not base64 decode!");
    let ciphertext = base64::decode(&payload.c).expect("Could not base64 decode!");
    match locker::register_finish(id, email, &input, &ciphertext) {
        Ok(response) => Ok(json!({ "id": response.id, "o": response.output })),
        Err(err) => {
            println!("Error in register_locker_finish: {:?}", err);
            Err(err)
            // json!({ "id": err.id, "o": err.msg })
        }
    }
}

#[post("/locker/open/start", format = "json", data = "<payload>")]
pub fn open_locker_start(
    payload: Json<OpenLockerStart>,
    _auth: Authenticated,
) -> Result<JsonValue, ApiError> {
    //TODO use _auth.session_key in order to decrypt payload and encrypt response
    let locker_id = payload.id.as_str();
    let email = payload.e.as_str();
    let input = base64::decode(&payload.i).expect("Could not base64 decode!");
    match locker::open_start(locker_id, email, &input) {
        Ok(response) => Ok(json!({ "id": response.id, "o": response.output, "n": response.nonce })),
        Err(err) => {
            println!("Error in open_locker_start: {:?}", err);
            Err(err)
            // json!({ "id": err.id, "o": err.msg, "n": err.nonce })
        }
    }
}

#[post("/locker/open/finish", format = "json", data = "<payload>")]
pub fn open_locker_finish(
    payload: Json<OpenLockerFinish>,
    _auth: Authenticated,
) -> Result<JsonValue, ApiError> {
    //TODO use _auth.session_key in order to decrypt payload and encrypt response
    let locker_id = &payload.id;
    let email = &payload.e;
    let input = base64::decode(&payload.i).expect("Could not base64 decode!");
    let nonce = payload.n;
    match locker::open_finish(locker_id, email, &input, nonce) {
        Ok(response) => Ok(json!({ "id": response.id, "o": response.output, "n": response.nonce })),
        Err(err) => {
            println!("Error in open_locker_finish: {:?}", err);
            Err(err)
        }
    }
}

#[post("/locker/delete/start", format = "json", data = "<payload>")]
pub fn delete_locker_start(
    payload: Json<DeleteLockerStart>,
    _auth: Authenticated,
) -> Result<JsonValue, ApiError> {
    //TODO use _auth.session_key in order to decrypt payload and encrypt response
    let locker_id = payload.id.as_str();
    let email = payload.e.as_str();
    let input = base64::decode(&payload.i).expect("Could not base64 decode!");
    match locker::delete_start(locker_id, email, &input) {
        Ok(response) => Ok(json!({ "id": response.id, "o": response.output, "n": response.nonce })),
        Err(err) => {
            println!("Error in delete_locker_start: {:?}", err);
            Err(err)
        }
    }
}

#[post("/locker/delete/finish", format = "json", data = "<payload>")]
pub fn delete_locker_finish(
    payload: Json<DeleteLockerFinish>,
    _auth: Authenticated,
) -> Result<JsonValue, ApiError> {
    //TODO use _auth.session_key in order to decrypt payload and encrypt response
    let locker_id = &payload.id;
    let email = &payload.e;
    let input = base64::decode(&payload.i).expect("Could not base64 decode!");
    let nonce = payload.n;
    match locker::delete_finish(locker_id, email, &input, nonce) {
        Ok(response) => Ok(json!({ "id": response.id, "o": response.output, "n": response.nonce })),
        Err(err) => {
            println!("Error in delete_locker_finish: {:?}", err);
            Err(err)
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
