use sha2::{Digest, Sha256};

pub fn hash_pin(pin: &str, salt: &str) -> String {
    let mut h = Sha256::new();
    h.update(salt.as_bytes());
    h.update(pin.as_bytes());
    hex(&h.finalize())
}

pub fn verify_pin(pin: &str, salt: &str, expected_hash: &str) -> bool {
    hash_pin(pin, salt) == expected_hash
}

/// 16 random bytes as hex, for a per-PIN salt.
pub fn new_salt() -> String {
    let mut b = [0u8; 16];
    let _ = getrandom::getrandom(&mut b);
    hex(&b)
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{:02x}", b)).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn hash_is_deterministic_and_salted() {
        assert_eq!(hash_pin("1234", "aa"), hash_pin("1234", "aa"));
        assert_ne!(hash_pin("1234", "aa"), hash_pin("1234", "bb"));
        assert_ne!(hash_pin("1234", "aa"), hash_pin("9999", "aa"));
    }
    #[test]
    fn verify_accepts_correct_rejects_wrong() {
        let salt = "deadbeef";
        let h = hash_pin("4242", salt);
        assert!(verify_pin("4242", salt, &h));
        assert!(!verify_pin("0000", salt, &h));
    }
    #[test]
    fn new_salt_is_32_hex_chars() {
        assert_eq!(new_salt().len(), 32);
    }
}
