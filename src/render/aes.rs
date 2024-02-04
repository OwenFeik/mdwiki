use std::fmt::Display;

use aes_gcm::{
    aead::{Aead, OsRng},
    AeadCore, Aes256Gcm, KeyInit,
};
use base64::Engine;

use crate::log;

fn warn(err: impl Display) {
    log::warning(err)
}

fn b64_encode(data: &[u8]) -> String {
    base64::engine::general_purpose::STANDARD.encode(data)
}

fn cipher(key: &str) -> Aes256Gcm {
    let mut key_bytes: [u8; 32] = [0; 32];
    for (u, v) in key_bytes.iter_mut().zip(key.as_bytes().iter()) {
        *u = *v;
    }

    Aes256Gcm::new(&key_bytes.into())
}

pub fn encrypt(plaintext: &str, key: &str) -> Result<(String, String), ()> {
    let cipher = cipher(key);
    let nonce = Aes256Gcm::generate_nonce(&mut OsRng);
    let ciphertext = cipher.encrypt(&nonce, plaintext.as_bytes()).map_err(warn)?;
    Ok((b64_encode(&nonce), b64_encode(&ciphertext)))
}

#[cfg(test)]
mod test {
    use super::*;

    fn b64_decode(data: &str) -> Result<Vec<u8>, ()> {
        base64::engine::general_purpose::STANDARD
            .decode(data)
            .map_err(warn)
    }

    fn decrypt(nonce: &str, ciphertext: &str, key: &str) -> Result<String, ()> {
        let cipher = cipher(key);
        let nonce_bytes: &[u8] = &b64_decode(nonce)?;
        let nonce: &[u8; 12] = nonce_bytes.try_into().map_err(warn)?;
        let data: &[u8] = &b64_decode(ciphertext)?;
        let plain_bytes = cipher.decrypt(nonce.into(), data).map_err(warn)?;
        let plaintext = String::from_utf8_lossy(&plain_bytes);
        Ok(plaintext.to_string())
    }

    #[test]
    fn test_aes_encode_decode() {
        let plaintext = "plaintext";
        let key = "mysupersafepassword";
        let (nonce, ciphertext) = encrypt(plaintext, key).unwrap();
        assert_ne!(ciphertext, plaintext);
        assert_eq!(decrypt(&nonce, &ciphertext, key).unwrap(), plaintext);
    }

    #[test]
    fn test_multiple_encode() {
        let plaintext = "<h1>Super secret section!</h1>";
        let key1 = "key1";
        let key2 = "key2";
        let key3 = "key3";

        // ciphertext = encode(encode(encode(plaintext, key1), key2), key3)
        let (nonce1, ciphertext) = encrypt(plaintext, key1).unwrap();
        let (nonce2, ciphertext) = encrypt(&ciphertext, key2).unwrap();
        let (nonce3, ciphertext) = encrypt(&ciphertext, key3).unwrap();

        // decoded = decode(decode(decode(ciphertext, key1), key2), key3)
        let decoded = decrypt(&nonce3, &ciphertext, key3).unwrap();
        let decoded = decrypt(&nonce2, &decoded, key2).unwrap();
        let decoded = decrypt(&nonce1, &decoded, key1).unwrap();

        assert_eq!(decoded, plaintext);
    }
}
