mod aes;

// export aes128,aes192,aes256 for aes encrypt and decrypt
pub use aes::{Aes128Crypto, Aes192Crypto, Aes256Crypto, AesCrypto};

#[test]
fn test_aes256_crypto() {
    let key = Aes256Crypto::generate_key();
    let iv = Aes256Crypto::generate_iv();
    let c = Aes256Crypto::new(&key, &iv);
    let s = "hello world";
    let encrypted = c.encrypt(s).unwrap();
    let decrypted = c.decrypt(&encrypted).unwrap();
    println!("encrypted: {}", encrypted);
    println!("decrypted: {}", decrypted);
    assert_eq!(s, decrypted);

    let key = AesCrypto::<aes::Aes256>::generate_key();
    let iv = AesCrypto::<aes::Aes256>::generate_iv();
    let c = AesCrypto::<aes::Aes256>::new(&key, &iv);
    let s = "hello world";
    let encrypted = c.encrypt(s).unwrap();
    let decrypted = c.decrypt(&encrypted).unwrap();
    println!("encrypted: {}", encrypted);
    println!("decrypted: {}", decrypted);
    assert_eq!(s, decrypted);
}
