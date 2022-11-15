mod init;
mod opaque;

pub use init::init;
pub use opaque::*;

use chacha20poly1305::aead::{Aead, NewAead};
use chacha20poly1305::{ChaCha20Poly1305, Key, Nonce};
use opaque_ke::rand::rngs::OsRng;
use opaque_ke::rand::RngCore;

// Given a key and plaintext, produce an AEAD ciphertext along with a nonce
pub fn encrypt_locker(key: &[u8], plaintext: &[u8]) -> Vec<u8> {
    let cipher = ChaCha20Poly1305::new(Key::from_slice(&key[..32]));

    let mut rng = OsRng;
    let mut nonce_bytes = [0u8; 12];
    rng.fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);

    let ciphertext = cipher.encrypt(nonce, plaintext.as_ref()).unwrap();
    [nonce_bytes.to_vec(), ciphertext].concat()
}

pub fn create_nonce() -> u32 {
    rand::random::<u32>()
}
