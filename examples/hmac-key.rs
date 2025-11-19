use rand::TryRngCore;

fn main() {
    use rand::rngs::OsRng;

    let mut key = [0u8; 32]; // 256-bit key for SHA-256 HMAC
    OsRng.try_fill_bytes(&mut key).unwrap();

    let key = key.iter().fold(String::new(), |mut acc, byte| {
        use core::fmt::Write;
        write!(acc, "{:02x}", byte).unwrap();
        acc
    });

    println!("HMAC_SECRET_KEY={}", key);
}
