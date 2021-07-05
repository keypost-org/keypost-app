// #[macro_use] extern crate lazy_static;

use std::collections::HashMap;
use std::sync::Mutex;

// https://github.com/SergioBenitez/Rocket/blob/v0.4.10/examples/uuid/src/main.rs
lazy_static! {
    static ref CACHE: Mutex<HashMap<u32, Vec<u8>>> = {
        let map = HashMap::new();
        Mutex::new(map)
    };
    static ref USERS: Mutex<HashMap<&'static str, Vec<u8>>> = {
        let map = HashMap::new();
        Mutex::new(map)
    };
}

pub fn insert(k: u32, v: Vec<u8>) -> bool {
    let mut cache = CACHE.lock().unwrap();
    cache.insert(k, v).is_some()
}

pub fn get(k: &u32) -> Option<Vec<u8>> {
    let cache = CACHE.lock().unwrap();
    cache.get(k).cloned()
}
