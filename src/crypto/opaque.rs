use std::sync::Mutex;

use opaque_ke::errors::ProtocolError;
use opaque_ke::{
    ciphersuite::CipherSuite, rand::rngs::OsRng, CredentialFinalization, CredentialRequest,
    Identifiers, RegistrationRequest, RegistrationUpload, ServerLogin, ServerLoginStartParameters,
    ServerLoginStartResult, ServerRegistration, ServerRegistrationStartResult, ServerSetup,
};

use crate::cache;
use crate::crypto;
use crate::persistence;
use crate::util;

lazy_static! {
    static ref SERVER_SETUP: Mutex<ServerSetup<DefaultCipherSuite>> = {
        let server_setup_location = util::default_dir() + "/server_setup.private";
        let server_setup = match util::read_file(&server_setup_location) {
            Ok(bytes) => {
                println!("DEBUG: Found server_setup file");
                ServerSetup::<DefaultCipherSuite>::deserialize(&bytes).unwrap_or_else(|err| {
                    println!("ERROR: {:?}", err);
                    panic!(
                        "Could not deserialize bytes from file {}",
                        &server_setup_location
                    )
                })
            }
            Err(err) => {
                println!("DEBUG: Could not find server_setup file - error: {:?}", err);
                let mut server_rng = OsRng;
                let server_setup = ServerSetup::<DefaultCipherSuite>::new(&mut server_rng);
                util::write_to_file(&server_setup_location, &server_setup.serialize())
                    .unwrap_or_else(|err| {
                        println!("ERROR: {:?}", err);
                        panic!(
                            "Could not write server_setup file to {}",
                            &server_setup_location
                        )
                    });
                server_setup
            }
        };
        Mutex::new(server_setup)
    };
}

// The CipherSuite trait allows to specify the underlying primitives
// that will be used in the OPAQUE protocol
pub struct DefaultCipherSuite;

#[cfg(feature = "ristretto255")]
impl CipherSuite for DefaultCipherSuite {
    type OprfCs = opaque_ke::Ristretto255;
    type KeGroup = opaque_ke::Ristretto255;
    type KeyExchange = opaque_ke::key_exchange::tripledh::TripleDh;
    type Ksf = opaque_ke::ksf::Identity;
}

#[cfg(not(feature = "ristretto255"))]
impl CipherSuite for DefaultCipherSuite {
    type OprfCs = p256::NistP256;
    type KeGroup = p256::NistP256;
    type KeyExchange = opaque_ke::key_exchange::tripledh::TripleDh;
    type Ksf = opaque_ke::ksf::Identity;
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
    ) -> ServerRegistrationStartResult<DefaultCipherSuite> {
        let registration_request_bytes =
            base64::decode(registration_request_base64).expect("Could not perform base64 decode");
        let server_setup = SERVER_SETUP.lock().unwrap(); // FIXME https://doc.rust-lang.org/stable/std/sync/struct.Mutex.html#poisoning
        ServerRegistration::<DefaultCipherSuite>::start(
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
            RegistrationUpload::<DefaultCipherSuite>::deserialize(&client_message_bytes[..])
                .unwrap(),
        );
        password_file.serialize().to_vec()
    }

    pub fn login_start(
        &self,
        email: &str,
        password_file_bytes: &[u8],
        credential_request_base64: &str,
    ) -> ServerLoginStartResult<DefaultCipherSuite> {
        let credential_request_bytes =
            base64::decode(credential_request_base64).expect("Could not perform base64 decode");
        let password_file =
            ServerRegistration::<DefaultCipherSuite>::deserialize(password_file_bytes).unwrap();
        let mut server_rng = OsRng;
        let server_setup = SERVER_SETUP.lock().unwrap();
        ServerLogin::start(
            &mut server_rng,
            &server_setup,
            Some(password_file),
            CredentialRequest::deserialize(&credential_request_bytes[..]).unwrap(),
            email.as_bytes(),
            ServerLoginStartParameters {
                context: None,
                identifiers: Identifiers {
                    client: None,
                    server: None,
                },
            },
        )
        .unwrap()
    }

    pub fn login_finish(
        &self,
        server_login_bytes: &[u8],
        credential_finalization_base64: &str,
    ) -> Result<(), ProtocolError> {
        let credential_finalization_bytes = base64::decode(credential_finalization_base64)
            .expect("Could not perform base64 deocde");
        let server_login =
            ServerLogin::<DefaultCipherSuite>::deserialize(server_login_bytes).unwrap();
        server_login.finish(CredentialFinalization::deserialize(
            &credential_finalization_bytes[..],
        )?)?;
        Ok(())
    }

    pub fn register_locker_start(
        &self,
        locker_id: &str,
        registration_request_bytes: &[u8],
    ) -> Result<String, ProtocolError> {
        let server_setup = SERVER_SETUP.lock().unwrap();
        let server_registration_start_result = ServerRegistration::<DefaultCipherSuite>::start(
            &server_setup,
            RegistrationRequest::deserialize(registration_request_bytes)?,
            locker_id.as_bytes(),
        )
        .unwrap();
        let registration_response_bytes = server_registration_start_result
            .message
            .serialize()
            .to_vec();
        Ok(base64::encode(registration_response_bytes))
    }

    pub fn register_locker_finish(
        &self,
        locker_id: &str,
        email: &str,
        message: &[u8],
        ciphertext: &[u8],
    ) -> Result<(), ProtocolError> {
        let server_registration: ServerRegistration<DefaultCipherSuite> =
            ServerRegistration::finish(RegistrationUpload::<DefaultCipherSuite>::deserialize(
                message,
            )?);
        let password_file = server_registration.serialize().to_vec();
        persistence::store_locker_contents(email, locker_id, &password_file, ciphertext)
            .unwrap_or_else(|err| {
                panic!("Could not store locker {} contents: {:?}", locker_id, err)
            });
        Ok(())
    }

    pub fn open_locker_start(
        &self,
        locker_id: &str,
        credential_request_bytes: &[u8],
        locker_password_file: &[u8],
        nonce: u32,
    ) -> Result<String, ProtocolError> {
        let server_setup = SERVER_SETUP.lock().unwrap();
        let password_file =
            ServerRegistration::<DefaultCipherSuite>::deserialize(locker_password_file).unwrap();
        let mut server_rng = OsRng;
        let server_login_start_result: ServerLoginStartResult<DefaultCipherSuite> =
            ServerLogin::start(
                &mut server_rng,
                &server_setup,
                Some(password_file),
                CredentialRequest::deserialize(credential_request_bytes).unwrap(),
                locker_id.as_bytes(),
                ServerLoginStartParameters::default(),
            )
            .unwrap_or_else(|_| {
                panic!(
                    "Could not execute ServerLogin::start: {:?}, {:?}",
                    locker_id, nonce
                )
            });
        let credential_response_bytes = server_login_start_result.message.serialize().to_vec();
        cache::insert(nonce, server_login_start_result.state.serialize().to_vec());
        Ok(base64::encode(credential_response_bytes))
    }

    pub fn open_locker_finish(
        &self,
        locker_contents: &[u8], // same as ciphertext
        credential_finalization_bytes: &[u8],
        server_login_bytes: &[u8],
    ) -> Result<Vec<u8>, ProtocolError> {
        let server_login_state =
            ServerLogin::<DefaultCipherSuite>::deserialize(server_login_bytes)?;
        let server_login_finish_result = server_login_state.finish(
            CredentialFinalization::deserialize(credential_finalization_bytes)?,
        )?;

        // Server sends locker contents, encrypted under the session key, to the client
        let encrypted_locker_contents =
            crypto::encrypt_locker(&server_login_finish_result.session_key, locker_contents);

        Ok(encrypted_locker_contents)
    }
}

impl std::default::Default for Opaque {
    fn default() -> Self {
        Self::new()
    }
}
