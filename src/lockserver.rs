extern crate openssl;
extern crate hex;
 
use openssl::sha;

fn main() {
    println!("Starting lock server");
    let mut hasher = sha::Sha256::new();
 
    hasher.update(b"Hello, ");
    hasher.update(b"world");
 
    let hash = hasher.finish();
    println!("Hashed \"Hello, world\" to {}", hex::encode(hash));
}
