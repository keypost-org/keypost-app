use std::env;
use std::fs;
use std::io::Error;

pub fn default_dir() -> String {
    String::from(env!("HOME")) + "/.keypost-app"
}

pub fn get_env_var(var: &str, default: &str) -> String {
    match env::var(var) {
        Ok(value) => value,
        Err(_e) => String::from(default),
    }
}

pub fn create_directory(dir: &str) -> Result<(), Error> {
    match fs::read_dir(dir) {
        Ok(_) => Ok(()),
        Err(err) => {
            println!("DEBUG: Looked for directory {}. But error {:?}", dir, err);
            println!("INFO: Will attempt to create directory {}", dir);
            fs::create_dir(dir)
        }
    }
}

pub fn write_to_file(file_path: &str, bytes: &[u8]) -> Result<(), Error> {
    fs::write(file_path, bytes)
}

pub fn read_file(file_path: &str) -> Result<Vec<u8>, Error> {
    fs::read(file_path).map_err(|err| {
        println!("DEBUG: Could not read file {}. Error: {:?}", file_path, err);
        err
    })
}
