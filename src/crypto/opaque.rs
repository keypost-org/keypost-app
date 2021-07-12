use std::sync::Mutex;

use curve25519_dalek::ristretto::RistrettoPoint;
use opaque_ke::{
    ciphersuite::CipherSuite, rand::rngs::OsRng, RegistrationRequest, RegistrationUpload,
    ServerLoginStartResult, ServerRegistration, ServerRegistrationStartResult, ServerSetup,
};
use opaque_ke::{
    CredentialFinalization, CredentialRequest, ServerLogin, ServerLoginStartParameters,
};

lazy_static! {
    //FIXME Generate this once and persist over restarts.
    //Use https://docs.rs/opaque-ke/0.6.0/opaque_ke/struct.ServerSetup.html#method.serialize (and deserialzie) at startup
    static ref SERVER_SETUP: Mutex<ServerSetup<Default>> = {
        let mut server_rng = OsRng;
        let server_setup = ServerSetup::<Default>::new(&mut server_rng);
        Mutex::new(server_setup)
    };
}

// The ciphersuite trait allows to specify the underlying primitives
// that will be used in the OPAQUE protocol
#[allow(dead_code)]
pub struct Default;
impl CipherSuite for Default {
    type Group = RistrettoPoint;
    type KeyExchange = opaque_ke::key_exchange::tripledh::TripleDH;
    type Hash = sha2::Sha512;
    type SlowHash = opaque_ke::slow_hash::NoOpHash;
}

pub struct Opaque {}

impl Opaque {
    pub fn new() -> Opaque {
        Opaque {}
    }

    pub fn server_side_registration_start(
        &self,
        registration_request_base64: &str,
        email: &str,
    ) -> ServerRegistrationStartResult<Default> {
        let registration_request_bytes =
            base64::decode(registration_request_base64).expect("Could not perform base64 decode");
        let server_setup = SERVER_SETUP.lock().unwrap();
        ServerRegistration::<Default>::start(
            &server_setup,
            RegistrationRequest::deserialize(&registration_request_bytes[..]).unwrap(),
            email.as_bytes(),
        )
        .unwrap()
    }

    pub fn server_side_registration_finish(&self, client_message_base64: &str) -> Vec<u8> {
        let client_message_bytes =
            base64::decode(client_message_base64).expect("Could not perform base64 decode");
        let password_file = ServerRegistration::finish(
            RegistrationUpload::<Default>::deserialize(&client_message_bytes[..]).unwrap(),
        );
        password_file.serialize()
    }

    pub fn login_start(
        &self,
        email: &str,
        password_file_bytes: &[u8],
        credential_request_base64: &str,
    ) -> ServerLoginStartResult<Default> {
        let credential_request_bytes =
            base64::decode(credential_request_base64).expect("Could not perform base64 decode");
        let password_file =
            ServerRegistration::<Default>::deserialize(&password_file_bytes).unwrap();
        let mut server_rng = OsRng;
        let server_setup = SERVER_SETUP.lock().unwrap();
        ServerLogin::start(
            &mut server_rng,
            &server_setup,
            Some(password_file),
            CredentialRequest::deserialize(&credential_request_bytes[..]).unwrap(),
            &email.as_bytes(),
            ServerLoginStartParameters::default(),
        )
        .unwrap()
    }

    pub fn login_finish(
        &self,
        server_login_bytes: &[u8],
        credential_finalization_base64: &str,
    ) -> Vec<u8> {
        let credential_finalization_bytes = base64::decode(credential_finalization_base64)
            .expect("Could not perform base64 deocde");
        let server_login = ServerLogin::<Default>::deserialize(server_login_bytes).unwrap();
        let server_login_finish_result = server_login
            .finish(
                CredentialFinalization::deserialize(&credential_finalization_bytes[..]).unwrap(),
            )
            .unwrap();
        server_login_finish_result.session_key
    }
}

impl std::default::Default for Opaque {
    fn default() -> Self {
        Self::new()
    }
}
