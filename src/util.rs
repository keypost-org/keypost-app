use std::env;
use std::fs;

pub fn default_dir() -> String {
    String::from(env!("HOME")) + "/.keypost-app"
}

pub fn get_env_var(var: &str, default: &str) -> String {
    match env::var(var) {
        Ok(value) => value,
        Err(_e) => String::from(default),
    }
}

pub fn create_directory(dir: &str) -> Result<(), std::io::Error> {
    match fs::read_dir(dir) {
        Ok(_) => Ok(()),
        Err(err) => {
            println!("WARNING: Looked for directory {}. But error {:?}", dir, err);
            println!("INFO: Will attempt to create directory {}", dir);
            fs::create_dir(dir)
        }
    }
}
