use std::collections::HashMap;
use std::sync::Mutex;

// https://github.com/SergioBenitez/Rocket/blob/v0.4.10/examples/uuid/src/main.rs
lazy_static! {
    static ref CACHE: Mutex<HashMap<u32, Vec<u8>>> = {
        let map = HashMap::new();
        Mutex::new(map)
    };
    static ref USERS: Mutex<HashMap<String, Vec<u8>>> = {
        let map = HashMap::new();
        Mutex::new(map)
    };
}

pub fn insert(k: u32, v: Vec<u8>) {
    let mut cache = CACHE.lock().unwrap();
    cache.insert(k, v);
}

pub fn get(k: &u32) -> Option<Vec<u8>> {
    let cache = CACHE.lock().unwrap();
    cache.get(k).cloned()
}

pub fn add_user(user_name: String, password_file: Vec<u8>) {
    let mut cache = USERS.lock().unwrap();
    cache.insert(user_name, password_file);
}

pub fn get_user(user_name: &str) -> Option<Vec<u8>> {
    let cache = USERS.lock().unwrap();
    cache.get(user_name).cloned()
}

pub fn user_exists(user_name: &str) -> bool {
    let cache = USERS.lock().unwrap();
    cache.contains_key(user_name)
}
