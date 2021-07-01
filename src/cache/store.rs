// #[macro_use] extern crate lazy_static;

use std::collections::HashMap;
use std::sync::Mutex;

// https://github.com/SergioBenitez/Rocket/blob/v0.4.10/examples/uuid/src/main.rs
lazy_static! {
    static ref CACHE: Mutex<HashMap<u32, Vec<u8>>> = {
        let map = HashMap::new();
        Mutex::new(map)
    };
}
pub struct Store {
    _cache: Option<HashMap<u32, Vec<u8>>>,
}

impl Store {
    pub fn new() -> Store {
        //FIXME rename this method (acts more like a handle) and remove _cache impl.
        let _cache = HashMap::<u32, Vec<u8>>::new();
        Store { _cache: None }
    }

    pub fn insert(self, k: u32, v: Vec<u8>) -> bool {
        let mut cache = CACHE.lock().unwrap();
        cache.insert(k, v).is_some()
    }

    pub fn get(&self, k: &u32) -> Option<Vec<u8>> {
        let cache = CACHE.lock().unwrap();
        cache.get(k).cloned()
    }
}

impl std::default::Default for Store {
    fn default() -> Self {
        Self::new()
    }
}
