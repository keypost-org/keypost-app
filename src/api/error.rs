use base64::DecodeError;
use rocket::http::{ContentType, Status};
use rocket::request::Request;
use rocket::response::{self, Responder, Response};
use std::io::Cursor;
use thiserror::Error;

// These errors are expected to use throughout the entire app, not just for api so that no lib specific errors are leaked out.
#[derive(Error, Debug)]
pub enum ApiError {
    #[error("Error during login: `{0}`")]
    LoginError(String),

    #[error("User not Authenticated.")]
    NotAuthenticated,

    #[error("Confirmation key `{0}` is invalid.")]
    InvalidConfirmationKey(String),

    #[error("Invalid API request: Expected {expected:?}.")]
    InvalidRequest { expected: String },

    #[error("Bad API request.")]
    BadRequest,

    #[error("Bad API request, decode error.")]
    BadRequestDecode(#[from] DecodeError),

    //FIXME? This errors with "method cannot be called on `&ProtocolError` due to unsatisfied trait bounds".
    //#[error("Bad API request, protocol error.")]
    //BadRequestProtocolFIXME(#[from] opaque_ke::errors::ProtocolError),
    //Using this instead of above for now.
    #[error("Bad API request, protocol error.")]
    BadRequestProtocol,

    #[error("Bad confirmation key or wrong email given.")]
    BadConfirmationKeyOrWrongEmail,

    #[error("Could not find key `{0}`")]
    LockerNotFound(String),

    #[error("Unknown locker error: `{0}`")]
    UnknownLockerError(String),

    #[error("Server error.")]
    // #[response(status = 500)]
    ServerError,

    #[error("Unknown API Error")]
    // #[response(status = 500)]
    UnknownError,
}

impl<'r> Responder<'r> for ApiError {
    fn respond_to(self, _: &Request) -> response::Result<'r> {
        Response::build()
            .sized_body(Cursor::new(format!("{:?}", self)))
            .header(ContentType::JSON)
            .status(get_status(self))
            .ok()
    }
}

fn get_status(err: ApiError) -> Status {
    match err {
        ApiError::LoginError(_) => Status::BadRequest,
        ApiError::NotAuthenticated => Status::Unauthorized,
        ApiError::InvalidConfirmationKey(_) => Status::BadRequest,
        ApiError::InvalidRequest { .. } => Status::BadRequest,
        ApiError::BadRequest => Status::BadRequest,
        ApiError::BadRequestDecode(_) => Status::BadRequest,
        ApiError::BadRequestProtocol => Status::BadRequest,
        ApiError::BadConfirmationKeyOrWrongEmail => Status::BadRequest,
        ApiError::LockerNotFound(_) => Status::NotFound,
        ApiError::UnknownLockerError(_) => Status::InternalServerError,
        ApiError::ServerError => Status::InternalServerError,
        ApiError::UnknownError => Status::InternalServerError,
    }
}
