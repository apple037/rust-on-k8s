use sha256;

pub fn hash_sha256(input: &str) -> String {
    sha256::digest(input.as_bytes())
}