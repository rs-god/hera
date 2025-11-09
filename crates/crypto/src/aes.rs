use base64::{Engine as _, engine::general_purpose::STANDARD};
use cbc::cipher::block_padding::Pkcs7;
use cbc::cipher::{
    BlockCipher, BlockDecrypt, BlockDecryptMut, BlockEncrypt, BlockEncryptMut, KeyInit, KeyIvInit,
};
use cbc::{Decryptor, Encryptor};
use rand::Rng;

const HEX_CHARS: &[u8] = b"0123456789abcdefABCDEF";

pub struct AesCrypto<T> {
    key: Vec<u8>,
    iv: Vec<u8>,
    _phantom: std::marker::PhantomData<T>,
}

fn generate_random_hex_string(length: usize) -> String {
    let mut rng = rand::rng();
    let mut result = String::with_capacity(length);

    for _ in 0..length {
        let idx = rng.random_range(0..HEX_CHARS.len());
        result.push(HEX_CHARS[idx] as char);
    }

    result
}

impl<T> AesCrypto<T>
where
    T: BlockCipher + BlockDecrypt + BlockEncrypt + KeyInit,
{
    pub fn new(key: &str, iv: &str) -> Self {
        Self {
            key: key.as_bytes().into(),
            iv: iv.as_bytes().into(),
            _phantom: std::marker::PhantomData,
        }
    }

    pub fn encrypt(&self, data: &str) -> Result<String, String> {
        let cipher = Encryptor::<T>::new_from_slices(&self.key, &self.iv)
            .map_err(|e| format!("failed to create encryptor error: {}", e))?;

        let data = data.as_bytes();
        let mut buffer = vec![0u8; data.len() + 16];
        buffer[..data.len()].copy_from_slice(data);

        let ciphertext = cipher
            .encrypt_padded_mut::<Pkcs7>(&mut buffer, data.len())
            .map_err(|e| format!("failed to encrypt error: {}", e))?;

        Ok(STANDARD.encode(ciphertext.to_vec()))
    }

    pub fn decrypt(&self, ciphertext: &str) -> Result<String, String> {
        let ciphertext = STANDARD
            .decode(ciphertext)
            .map_err(|e| format!("failed to decode error: {}", e))?;
        let cipher = Decryptor::<T>::new_from_slices(&self.key, &self.iv)
            .map_err(|e| format!("failed to create decryptor error: {}", e))?;

        let mut buffer = ciphertext.to_vec();
        let plaintext = cipher
            .decrypt_padded_mut::<Pkcs7>(&mut buffer)
            .map_err(|e| format!("failed to decrypt error: {}", e))?;

        let s = String::from_utf8(plaintext.to_vec())
            .map_err(|e| format!("failed to convert error: {}", e))?;
        Ok(s)
    }

    pub fn generate_iv() -> String {
        generate_random_hex_string(16)
    }
}

pub use aes::{Aes128, Aes192, Aes256};
pub type Aes128Crypto = AesCrypto<Aes128>;
pub type Aes192Crypto = AesCrypto<Aes192>;
pub type Aes256Crypto = AesCrypto<Aes256>;

impl Aes128Crypto {
    pub fn generate_key() -> String {
        generate_random_hex_string(16)
    }
}

impl Aes192Crypto {
    pub fn generate_key() -> String {
        generate_random_hex_string(24)
    }
}

impl Aes256Crypto {
    pub fn generate_key() -> String {
        generate_random_hex_string(32)
    }
}

#[cfg(test)]
mod tests {
    use super::{Aes256Crypto, generate_random_hex_string};

    #[test]
    fn test_generate_random_hex_string() {
        for _ in 0..100 {
            let result = generate_random_hex_string(16);
            println!("{}", result);
            assert_eq!(result.len(), 16);
        }
    }

    #[test]
    fn test_aes() {
        let key = Aes256Crypto::generate_key();
        let iv = Aes256Crypto::generate_iv();
        let c = Aes256Crypto::new(&key, &iv);
        let s = "hello world";
        let encrypted = c.encrypt(s).unwrap();
        let decrypted = c.decrypt(&encrypted).unwrap();
        assert_eq!(s, decrypted);
    }
}
