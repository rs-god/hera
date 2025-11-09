use crypto::{Aes128Crypto, Aes192Crypto, Aes256Crypto};

fn main() {
    let key = Aes256Crypto::generate_key(); // you can also customize the key
    let iv = Aes256Crypto::generate_iv();
    let c = Aes256Crypto::new(&key, &iv);
    let s = "hello world";
    let encrypted = c.encrypt(s).unwrap();
    let decrypted = c.decrypt(&encrypted).unwrap();
    // assert_eq!(s, decrypted);
    println!("encrypted:{}", encrypted);
    println!("decrypted:{}", decrypted);

    let key = Aes192Crypto::generate_key(); // you can also customize the key
    let iv = Aes192Crypto::generate_iv();
    let c = Aes192Crypto::new(&key, &iv);
    let s = "hello world";
    let encrypted = c.encrypt(s).unwrap();
    let decrypted = c.decrypt(&encrypted).unwrap();
    // assert_eq!(s, decrypted);
    println!("encrypted:{}", encrypted);
    println!("decrypted:{}", decrypted);

    let key = Aes128Crypto::generate_key(); // you can also customize the key
    let iv = Aes128Crypto::generate_iv();
    let c = Aes128Crypto::new(&key, &iv);
    let s = "hello world";
    let encrypted = c.encrypt(s).unwrap();
    let decrypted = c.decrypt(&encrypted).unwrap();
    // assert_eq!(s, decrypted);
    println!("encrypted:{}", encrypted);
    println!("decrypted:{}", decrypted);
}
