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

fn b64_decode(data: &str) -> Result<Vec<u8>, ()> {
    base64::engine::general_purpose::STANDARD
        .decode(data)
        .map_err(warn)
}

fn cipher(key: &str) -> Aes256Gcm {
    let mut key_bytes: [u8; 32] = [0; 32];
    for (u, v) in key_bytes.iter_mut().zip(key.as_bytes().iter()) {
        *u = *v;
    }

    Aes256Gcm::new(&key_bytes.into())
}

fn encode(plaintext: &str, key: &str) -> Result<(String, String), ()> {
    let cipher = cipher(key);
    let nonce = Aes256Gcm::generate_nonce(&mut OsRng);
    let ciphertext = cipher.encrypt(&nonce, plaintext.as_bytes()).map_err(warn)?;
    Ok((b64_encode(&nonce), b64_encode(&ciphertext)))
}

fn decode(nonce: &str, ciphertext: &str, key: &str) -> Result<String, ()> {
    let cipher = cipher(key);
    let nonce_bytes: &[u8] = &b64_decode(nonce)?;
    let nonce: &[u8; 12] = nonce_bytes.try_into().map_err(warn)?;
    let data: &[u8] = &b64_decode(ciphertext)?;
    let plain_bytes = cipher.decrypt(nonce.into(), data).map_err(warn)?;
    let plaintext = String::from_utf8_lossy(&plain_bytes);
    Ok(plaintext.to_string())
}

#[cfg(test)]
mod test {
    use crate::render::html::aes::decode;

    use super::encode;

    #[test]
    fn test_aes_encode_decode() {
        let plaintext = "plaintext";
        let key = "mysupersafepassword";
        let (nonce, ciphertext) = encode(plaintext, key).unwrap();
        assert_eq!(decode(&nonce, &ciphertext, key).unwrap(), plaintext);
    }
}
