use std::collections::HashMap;
use std::sync::Mutex;

// https://github.com/SergioBenitez/Rocket/blob/v0.4.10/examples/uuid/src/main.rs
lazy_static! {
    static ref CACHE: Mutex<HashMap<u32, Vec<u8>>> = {
        let map = HashMap::new();
        Mutex::new(map)
    };
}

pub fn insert(k: u32, v: Vec<u8>) {
    let mut cache = CACHE.lock().unwrap();
    cache.insert(k, v);
}

pub fn insert_str(k: u32, s: &str) {
    let v = s.as_bytes().to_vec();
    let mut cache = CACHE.lock().unwrap();
    cache.insert(k, v);
}

pub fn get(k: &u32) -> Option<Vec<u8>> {
    let cache = CACHE.lock().unwrap();
    cache.get(k).cloned()
}

pub fn get_str(k: &u32) -> Option<String> {
    let cache = CACHE.lock().unwrap();
    cache.get(k).map(|v| String::from_utf8_lossy(v).to_string())
}
