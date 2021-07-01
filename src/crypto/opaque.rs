#![allow(unused_imports)]
use std::collections::HashMap;
use std::io::{Error, ErrorKind};
use std::process::exit;
use std::sync::Mutex;

use curve25519_dalek::ristretto::RistrettoPoint;
use opaque_ke::keypair::KeyPair;
use opaque_ke::ClientRegistrationStartResult;
use opaque_ke::{
    ciphersuite::CipherSuite, rand::rngs::OsRng, ClientRegistration,
    ClientRegistrationFinishParameters, RegistrationRequest, RegistrationResponse,
    RegistrationUpload, ServerLoginStartResult, ServerRegistration, ServerRegistrationStartResult,
};
use opaque_ke::{
    ClientLogin, ClientLoginFinishParameters, ClientLoginStartParameters, CredentialFinalization,
    CredentialRequest, CredentialResponse, ServerLogin, ServerLoginStartParameters,
};

lazy_static! {
    static ref KEY_PAIR: Mutex<KeyPair<RistrettoPoint>> = {
        let mut server_rng = OsRng;
        let key_pair = Default::generate_random_keypair(&mut server_rng);
        Mutex::new(key_pair)
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
    ) -> ServerRegistrationStartResult<Default> {
        let registration_request_bytes =
            base64::decode(registration_request_base64).expect("Could not perform base64 decode");
        let key_pair = KEY_PAIR.lock().unwrap();
        let server_public_key = key_pair.public();
        let mut server_rng = OsRng;
        ServerRegistration::<Default>::start(
            &mut server_rng,
            RegistrationRequest::deserialize(&registration_request_bytes[..]).unwrap(),
            server_public_key,
        )
        .unwrap()
    }

    pub fn server_side_registration_finish(
        &self,
        client_message_base64: &str,
        server_registration_bytes: &[u8],
    ) -> Vec<u8> {
        let client_message_bytes =
            base64::decode(client_message_base64).expect("Could not perform base64 decode");

        let server_registration =
            ServerRegistration::<Default>::deserialize(server_registration_bytes).unwrap();

        let password_file = server_registration
            .finish(RegistrationUpload::deserialize(&client_message_bytes[..]).unwrap())
            .unwrap();
        password_file.serialize()
    }
}

impl std::default::Default for Opaque {
    fn default() -> Self {
        Self::new()
    }
}
