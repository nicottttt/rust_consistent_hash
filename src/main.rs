extern crate twox_hash;
use twox_hash::XxHash64;
use std::hash::Hasher;
use rust_consistent_hash::consistent::Consistent;

fn key_mapping_test(){
    let mut c = Consistent::new_ring(15);
    c.add_server("Server1".to_string());
    c.add_server("Server2".to_string());
    c.add_server("Server3".to_string());
    c.add_server("Server4".to_string());

    let key1 = "key2222";
    let key2 = "key222222";
    let key3 = "key22";
    let key4 = "key2";


    let mut hasher = XxHash64::with_seed(0);
    hasher.write(key1.as_bytes()); // Write action is append, need to rephrase every time
    let hash1 = hasher.finish() % 1024;

    let mut hasher = XxHash64::with_seed(0);
    hasher.write(key2.as_bytes());
    let hash2 = hasher.finish() % 1024;

    let mut hasher = XxHash64::with_seed(0);
    hasher.write(key3.as_bytes());
    let hash3 = hasher.finish() % 1024;

    let mut hasher = XxHash64::with_seed(0);
    hasher.write(key4.as_bytes());
    let hash4 = hasher.finish() % 1024;

    println!("Key|{}|'s value is {}, and it is mapped to:{}", key1, hash1, c.map_key(&key1.to_string()));
    println!("Key|{}|'s value is {}, and it is mapped to:{}", key2, hash2, c.map_key(&key2.to_string()));
    println!("Key|{}|'s value is {}, and it is mapped to:{}", key3, hash3, c.map_key(&key3.to_string()));
    println!("Key|{}|'s value is {}, and it is mapped to:{}", key4, hash4, c.map_key(&key4.to_string()));
}

fn main() {
    key_mapping_test();
}