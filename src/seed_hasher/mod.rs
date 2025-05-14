use sha2::{Digest, Sha256};

fn generate_seed(input: String) -> i64 {
    // Try to parse the input string as an i64 directly
    if let Ok(parsed) = input.parse::<i64>() {
        return parsed;
    }

    // If parsing fails, hash the string
    let mut hasher = Sha256::new();
    hasher.update(input);
    let result = hasher.finalize();

    // Use the first 8 bytes of the hash to create an i64 seed
    let mut seed_bytes = [0u8; 8];
    seed_bytes.copy_from_slice(&result[0..8]);

    // Convert the byte array to an i64
    i64::from_be_bytes(seed_bytes)
}
