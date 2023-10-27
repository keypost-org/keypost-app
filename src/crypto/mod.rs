mod init;
mod opaque;

pub use init::*;
pub use opaque::*;

use chacha20poly1305::aead::{Aead, NewAead};
use chacha20poly1305::{ChaCha20Poly1305, Key, Nonce};
use opaque_ke::rand::rngs::OsRng;
use opaque_ke::rand::RngCore;

pub fn encrypt_bytes(nonce: &[u8], key: &[u8], plaintext: &[u8]) -> Vec<u8> {
    encrypt(&nonce[..12], key, plaintext).expect("Could not encrypt bytes!")
}

pub fn encrypt_bytes_with_u32_nonce(u32_nonce: &u32, key: &[u8], plaintext: &[u8]) -> Vec<u8> {
    let nonce = expand_u32_nonce(u32_nonce);
    encrypt_bytes(&nonce, key, plaintext)
}

// Given a key and plaintext, produce an AEAD ciphertext along with a nonce
pub fn encrypt_locker(key: &[u8], plaintext: &[u8]) -> Vec<u8> {
    let mut rng = OsRng;
    let mut nonce_bytes = [0u8; 12];
    rng.fill_bytes(&mut nonce_bytes);

    let ciphertext = encrypt(&nonce_bytes, key, plaintext).expect("Could not encrypt locker!");
    [nonce_bytes.to_vec(), ciphertext].concat()
}

pub fn create_nonce() -> u32 {
    rand::random::<u32>()
}

pub fn expand_u32_nonce(u32_nonce: &u32) -> Vec<u8> {
    [
        u32_nonce.to_be_bytes(),
        u32_nonce.to_be_bytes(),
        u32_nonce.to_be_bytes(),
    ]
    .concat()
}

pub fn rand_bytes() -> Vec<u8> {
    let mut rng = OsRng;
    let mut bytes = [0u8; 128];
    rng.fill_bytes(&mut bytes);
    bytes.to_vec()
}

fn encrypt(
    nonce_bytes: &[u8],
    key: &[u8],
    plaintext: &[u8],
) -> Result<Vec<u8>, chacha20poly1305::aead::Error> {
    let cipher = ChaCha20Poly1305::new(Key::from_slice(&key[..32]));
    let nonce = Nonce::from_slice(nonce_bytes);
    cipher.encrypt(nonce, plaintext.as_ref())
}
