use std::sync::Mutex;

use opaque_ke::errors::ProtocolError;
use opaque_ke::{
    ciphersuite::CipherSuite, rand::rngs::OsRng, CredentialFinalization, CredentialRequest,
    Identifiers, RegistrationRequest, RegistrationUpload, ServerLogin, ServerLoginStartParameters,
    ServerLoginStartResult, ServerRegistration, ServerRegistrationStartResult, ServerSetup,
};

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
}

impl std::default::Default for Opaque {
    fn default() -> Self {
        Self::new()
    }
}
