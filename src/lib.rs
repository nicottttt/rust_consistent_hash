pub mod consistent;

pub mod consistent_hash {
    include!(concat!(env!("OUT_DIR"), "/consistent_hash.rs"));
}