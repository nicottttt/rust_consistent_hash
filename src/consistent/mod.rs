extern crate twox_hash;

use std::collections::HashMap;
use twox_hash::XxHash64;
use std::hash::Hasher;

pub trait HasherTrait {
    fn hash_to_used(&self, data: &[u8]) -> u64;
}

pub struct MyHasher {}

impl HasherTrait for MyHasher {
    fn hash_to_used(&self, data: &[u8]) -> u64 {
        let mut hasher = XxHash64::with_seed(0);
        hasher.write(data);
        hasher.finish() % 1024
    }
}

pub struct Consistent {
    hasher: Box<dyn HasherTrait>,
    ring: HashMap<u64, String>,
    sorted_set: Vec<u64>,
    replication_factor: i32,
    mapping: HashMap<String, String>,
    serverlist: Vec<String>,
}

impl Consistent {
    pub fn new_ring(replication_factor: i32) -> Consistent {
        Consistent {
            hasher: Box::new(MyHasher {}),
            ring: HashMap::new(),
            sorted_set: Vec::new(),
            replication_factor,
            mapping: HashMap::new(),
            serverlist: Vec::new(),
        }
    }

    pub fn get_ring(&self) -> HashMap<u64, String> {
        self.ring.clone()
    }

    pub fn get_hasher(&self) -> &Box<dyn HasherTrait> {
        &self.hasher
    }

    pub fn get_mapping(&self) -> HashMap<String, String> {
        self.mapping.clone()
    }

    pub fn add_server(&mut self, server: String) {
        for i in 0..self.replication_factor {
            let key = format!("{}{}", server, i).into_bytes(); // Virtual node
            let h = self.hasher.hash_to_used(&key);
            self.ring.insert(h, server.clone());// maybe can use entry to avoid hash collistion?
            self.sorted_set.push(h);
        }

        self.sorted_set.sort();

        self.serverlist.push(server.clone());

        if self.mapping.is_empty() {
            return;
        }

        self.redirect_key_from_add_server(server);// give the ownership of server to the function
    }

    fn add_key(&mut self, key: String, server: String) {
        self.mapping.insert(key, server);
    }

    pub fn add_key_public(&mut self, key: String) {
        self.add_key(key.clone(), self.map_key(&key));
    }

    fn redirect_key_from_add_server(&mut self, server: String) {
        for (key, _) in self.mapping.clone() {
            if self.map_key(&key) == server {
                self.add_key(key, server.clone());
            }
        }
    }

    pub fn del_server(&mut self, server: String) {
        // Detect if there is this server or not
        match self.serverlist.iter().position(|x| *x == server) {
            Some(idx) => {
                self.serverlist.remove(idx);
                println!("Server '{}' was removed.", server);
            }
            None => {
                println!("Server '{}' was not found.", server);
                return;
            }
        }

        for i in 0..self.replication_factor {
            let key = format!("{}{}", server, i).into_bytes();// Virtual node
            let h = self.hasher.hash_to_used(&key);
            self.del_slice(h);
        }

        if self.mapping.is_empty() {
            return;
        }

        self.redirect_key_from_remove_server(server);
    }

    fn del_slice(&mut self, val: u64) {
        self.sorted_set.retain(|&x| x != val);
    }

    pub fn del_key(&mut self, key: String) {
        if self.mapping.contains_key(&key) {
            self.mapping.remove(&key);
        } else {
            println!("Key '{}' was not found.", key);
        }
    }

    fn redirect_key_from_remove_server(&mut self, server: String) {
        for (k, v) in self.mapping.clone() {
            if v == server {
                self.add_key_public(k);
            }
        }
    }

    pub fn map_key(&self, k: &String) -> String {
        let key = k.as_bytes();
        let hash = self.hasher.hash_to_used(key);
        // println!("hash is {}", hash);

        let idx = match self.sorted_set.binary_search(&hash) {// Search in the server hash
            Ok(idx) | Err(idx) => idx,
        };

        let hash_idx = if idx == 0 {
            self.sorted_set[0]
        } else if idx == self.sorted_set.len() {
            *self.sorted_set.last().unwrap()
        } else {
            self.sorted_set[idx]
        };

        self.ring[&hash_idx].clone()
    }

    pub fn traverse_hash_ring(&self) {
        for (hash, server) in &self.ring {
            println!("Server {}, hash {}", server, hash);
        }
    }

    pub fn traverse_sorted_set(&self) {
        for (i, hash) in self.sorted_set.iter().enumerate() {
            println!("Index {}, hash {}", i, hash);
        }
    }

    pub fn traverse_mapping(&self) {
        for (key, server) in &self.mapping {
            println!("Key {}, Server {}", key, server);
        }
    }

    pub fn traverse_serverlist(&self) {
        for server in &self.serverlist {
            println!("Server {}", server);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_del_key() {
        let mut c = Consistent::new_ring(15);
        c.add_server("Server1".to_string());
        c.add_server("Server2".to_string());
        c.add_server("Server3".to_string());
        c.add_server("Server4".to_string());
        let key = "key2222";
        c.add_key_public(key.to_string()); 

        assert!(c.mapping[key] == "Server4".to_string());

    }

}