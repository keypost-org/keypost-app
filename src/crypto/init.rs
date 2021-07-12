use crate::util;

pub fn init() -> Result<(), std::io::Error> {
    let app_dir = util::default_dir();
    util::create_directory(&app_dir)
}
