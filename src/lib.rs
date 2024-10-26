#![warn(clippy::all, rust_2018_idioms)]
mod dev_widgets;
pub use dev_widgets::{DevWidget, DevWidgetsDisplay, View};

mod app;
pub use app::FireApp;

use std::time::SystemTime;

#[derive(serde::Serialize, serde::Deserialize)]
pub struct AesPayload {
    ciphertext: Vec<u8>,
    nonce: [u8; 12],
    #[serde(with = "humantime_serde")]
    timestamp: SystemTime,
}

use base64::{engine::general_purpose::STANDARD, Engine as _};
use aes_gcm_siv::{
    aead::{Aead, KeyInit, OsRng},
    Aes256GcmSiv, Nonce
};
use argon2::{
    password_hash::{
        PasswordHash, PasswordHasher, PasswordVerifier, SaltString
    },
    Argon2
};

impl AesPayload {
    // -> (Nonce, Ciphertext)
    pub fn encrypt(aes_key_base64: String, text: String) -> ([u8; 12], Vec<u8>) {
        let aes_key = STANDARD.decode(aes_key_base64).unwrap(); // FIXME
        let cipher = Aes256GcmSiv::new_from_slice(&aes_key).unwrap(); // FIXME: handle errors
                                                                      // correctly
        let salt = SaltString::generate(&mut OsRng);
        let mut nonce = [0u8; 12];
        
        Argon2::default().hash_password_into(text.as_bytes(), salt.as_str().as_bytes(), &mut nonce).unwrap(); // FIXME^
        let nonce_cipher = Nonce::from_slice(&nonce);

        ( nonce, cipher.encrypt(nonce_cipher, text.as_ref()).unwrap() ) // FIXME
    }

    // -> (Timestamp as RFC 3339, Message)
    pub fn decrypt(aes_key_base64: String, payload_base64: String) -> (String, String) {
        let aes_key = STANDARD.decode(aes_key_base64).unwrap(); //FIXME
        let cipher = Aes256GcmSiv::new_from_slice(&aes_key).unwrap(); //FIXME
        
        // FIXME
        let payload: AesPayload = bincode::deserialize(
            // FIXME: what the fuck is going on
            &STANDARD.decode( &payload_base64[..] ).unwrap()[..]
        ).unwrap();

        let p_nonce = Nonce::from_slice(&payload.nonce);
        // FIXME
        let plaintext = cipher.decrypt(p_nonce, payload.ciphertext.as_ref()).unwrap();

        let mut timestamp_text = humantime::format_rfc3339(payload.timestamp.clone()).to_string();

        // FIXME
        ( timestamp_text, String::from_utf8(plaintext).unwrap() )
    }
}
