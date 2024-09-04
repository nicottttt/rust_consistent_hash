extern crate twox_hash;
use twox_hash::XxHash64;
use std::hash::Hasher;
use rust_consistent_hash::consistent::Consistent;
use rust_consistent_hash::consistent_hash::{MapkeyRequest};
use rust_consistent_hash::consistent_hash::{AddkeyRequest};
use rust_consistent_hash::consistent_hash::{RemoveServerRequest};
use rust_consistent_hash::consistent_hash::consistent_hash_client::ConsistentHashClient;
use uuid::Uuid;


fn register_server() -> Consistent {
    let mut c = Consistent::new_ring(15);
    c.add_server("Server1".to_string());
    c.add_server("Server2".to_string());
    c.add_server("Server3".to_string());
    c.add_server("Server4".to_string());
    c
}

async fn key_mapping_test() -> Result<(), Box<dyn std::error::Error>> {
    let c = register_server();

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
    Ok(())
}

async fn remove_server() -> Result<(), Box<dyn std::error::Error>> {
    let mut c = register_server();
    c.traverse_serverlist();

    c.del_server("Server4".to_string());
    c.traverse_serverlist();
    Ok(())
}

async fn server_key() -> Result<(), Box<dyn std::error::Error>> {
    let mut c = register_server();

    for i in 0..100{
        let key = format!("key{}", i);
        c.add_key_public(key);
    }

    c.traverse_mapping();

    c.del_server("Server2".to_string());
    c.traverse_mapping();
    Ok(())
}

async fn grpc_connection()  -> Result<(), Box<dyn std::error::Error>> {
    // gRPC client
    let channel = tonic::transport::Endpoint::new("http://[::1]:50052")?
        .connect()
        .await?;

    let mut client = ConsistentHashClient::new(channel); // async use here

    // local mapping by consistent hash 
    let c = register_server();

    let mut correct_mapping = 0;
    let mut wrong_mapping = 0;
    for _i in 0..100{
        // let str = format!("key{}", i);
        let str = Uuid::new_v4().to_string();
        let local_map = c.map_key(&str);
        let request = tonic::Request::new(MapkeyRequest {
            server: str.into(),
        });
        let response = client.key_map_server(request).await?;
        let remote_map = response.into_inner().result;
        // println!("RESPONSE={:?}", remote_map);
        
        if local_map == remote_map {
            correct_mapping += 1;
            //println!("Local map and remote map are the same");
        } else {
            wrong_mapping += 1;
            //println!("Local map and remote map are different");
        }
    }
    println!("Correct mapping: {}, Wrong mapping: {}", correct_mapping, wrong_mapping);

    Ok(())
}

async fn remove_server_grpc_test()  -> Result<(), Box<dyn std::error::Error>> {
    // gRPC client
    let channel = tonic::transport::Endpoint::new("http://[::1]:50052")?
        .connect()
        .await?;

    let mut client = ConsistentHashClient::new(channel); // async use here

    let mut c = register_server();

    // Add key to server
    for _i in 0..100{
        let str = Uuid::new_v4().to_string();
        c.add_key_public(str.clone()); // local
        let request = tonic::Request::new(AddkeyRequest {
            key: str.into(),
        });
        client.add_key(request).await?; // remote
    }

    // Del server
    let request_server = "Server1".to_string();
    c.del_server(request_server.clone()); // local

    let request = tonic::Request::new(RemoveServerRequest {
        server: request_server.into(),
    });
    let response = client.remove_server(request).await?; // remote
    let mapping = response.into_inner().result;

    if mapping == c.get_mapping() { // compare after remove server
        println!("Local mapping and remote mapping are the same");
    } else {
        println!("Local mapping and remote mapping are different");
    }

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let input = 4;
    match input {
        1 => key_mapping_test().await?,
        2 => remove_server().await?,
        3 => server_key().await?,
        4 => grpc_connection().await?,
        5 => remove_server_grpc_test().await?,
        _ => println!("Invalid input"),
    }

    Ok(())
}