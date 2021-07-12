use std::fs;

use crate::util;

pub fn init() -> Result<(), std::io::Error> {
    let app_dir = util::default_dir();
    match fs::read_dir(&app_dir) {
        Ok(_) => Ok(()),
        Err(err) => {
            println!(
                "WARNING: Looked for directory {}. But error {:?}",
                &app_dir, err
            );
            println!("INFO: Will attempt to create directory {}", &app_dir);
            fs::create_dir(&app_dir)
        }
    }
}
