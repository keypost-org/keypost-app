use crate::cache;
use crate::crypto;
use crate::persistence;

#[derive(Debug)]
pub struct LockerResponse {
    pub id: u32,
    pub output: String,
    pub nonce: u32,
}

#[derive(Debug)]
pub struct LockerError {
    pub id: u32,
    pub msg: String,
    pub nonce: u32,
}

pub fn register_start(id: &str, input: &[u8]) -> Result<LockerResponse, LockerError> {
    let opaque = crypto::Opaque::new();
    match opaque.register_locker_start(id, input) {
        Ok(output) => Ok(LockerResponse {
            id: 0,
            output,
            nonce: 0,
        }),
        Err(err) => {
            println!("Error in register_locker_start: {:?}", err);
            Err(LockerError {
                id: 0,
                msg: "There was an error during register_locker_start".to_string(),
                nonce: 0,
            })
        }
    }
}

pub fn register_finish(
    locker_id: &str,
    email: &str,
    input: &[u8],
    ciphertext: &[u8],
) -> Result<LockerResponse, LockerError> {
    let opaque = crypto::Opaque::new();
    match opaque.register_locker_finish(locker_id, email, input, ciphertext) {
        Ok(()) => Ok(LockerResponse {
            id: 0,
            output: "Success".to_string(),
            nonce: 0,
        }),
        Err(err) => {
            println!("Error in register_locker_finish: {:?}", err);
            Err(LockerError {
                id: 0,
                msg: "There was an error during register_locker_finish".to_string(),
                nonce: 0,
            })
        }
    }
}

pub fn open_start(
    locker_id: &str,
    email: &str,
    input: &[u8],
) -> Result<LockerResponse, LockerError> {
    let nonce: u32 = crypto::create_nonce();
    let (locker_psswd_file, _ciphertext) = {
        match persistence::fetch_locker_contents(email, locker_id) {
            Ok((l, c)) => (l, c),
            Err(err) => {
                println!("Error fetching locker contents: {:?}", err);
                return Err(LockerError {
                    id: 0,
                    msg: "There was an error during open_locker_start".to_string(),
                    nonce,
                });
            }
        }
    };
    let opaque = crypto::Opaque::new();
    match opaque.open_locker_start(locker_id, input, &locker_psswd_file, nonce) {
        Ok(output) => Ok(LockerResponse {
            id: 0,
            output,
            nonce,
        }),
        Err(err) => {
            println!("Error in open_locker_start: {:?}", err);
            Err(LockerError {
                id: 0,
                msg: "There was an error during open_locker_start".to_string(),
                nonce,
            })
        }
    }
}

pub fn open_finish(
    locker_id: &str,
    email: &str,
    input: &[u8],
    nonce: u32,
) -> Result<LockerResponse, LockerError> {
    let (_locker_psswd_file, ciphertext) = persistence::fetch_locker_contents(email, locker_id)
        .expect("Could not get locker_contents");
    let server_login_bytes: Vec<u8> = cache::get(&nonce)
        .unwrap_or_else(|| panic!("Could not find cached server_login_bytes: {:?}", nonce));
    let opaque = crypto::Opaque::new();
    match opaque.open_locker_finish(&ciphertext, input, &server_login_bytes) {
        Ok(encrypted_ciphertext) => Ok(LockerResponse {
            id: 0,
            output: base64::encode(encrypted_ciphertext),
            nonce,
        }),
        Err(err) => {
            println!("Error in open_locker_finish: {:?}", err);
            Err(LockerError {
                id: 0,
                msg: "There was an error during open_locker_finish".to_string(),
                nonce,
            })
        }
    }
}

pub fn delete_start(
    locker_id: &str,
    email: &str,
    input: &[u8],
) -> Result<LockerResponse, LockerError> {
    //Client to prove ownership (i.e. open_start accomplishes this) in order to allow them to call delete_finish().
    open_start(locker_id, email, input)
}

pub fn delete_finish(
    locker_id: &str,
    email: &str,
    input: &[u8],
    nonce: u32,
) -> Result<LockerResponse, LockerError> {
    //Finish the open-locker opaque protocol, but instead of returning the encrypted key, delete locker contents (i.e. key).
    match open_finish(locker_id, email, input, nonce) {
        Ok(_) => delete_contents(email, locker_id, nonce),
        Err(err) => {
            println!("Error in locker::delete_finish: {:?}", err);
            Err(LockerError {
                id: 0,
                msg: "There was an error during locker::delete_finish".to_string(),
                nonce,
            })
        }
    }
}

fn delete_contents(
    email: &str,
    locker_id: &str,
    nonce: u32,
) -> Result<LockerResponse, LockerError> {
    match persistence::delete_locker_contents(email, locker_id) {
        Ok(_) => Ok(LockerResponse {
            id: 0,
            output: "Key deleted!".to_string(),
            nonce,
        }),
        Err(err) => {
            println!("Error in locker::delete_contents: {:?}", err);
            Err(LockerError {
                id: 0,
                msg: "There was an error during locker::delete_contents".to_string(),
                nonce,
            })
        }
    }
}
