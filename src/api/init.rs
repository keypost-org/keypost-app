use crate::api::*;
use crate::persistence;

use rocket_contrib::serve::StaticFiles;

pub fn init() -> Result<(), std::io::Error> {
    rocket().launch();
    Ok(())
}

/// https://github.com/SergioBenitez/Rocket/tree/v0.4.10/examples
fn rocket() -> rocket::Rocket {
    persistence::get_active_users().expect("Could not get users!");

    rocket::ignite()
        .mount(
            "/",
            routes![
                register_start,
                register_finish,
                login_start,
                login_finish,
                login_verify,
                logout,
                register_locker_start,
                register_locker_finish,
                open_locker_start,
                open_locker_finish,
                delete_locker_start,
                delete_locker_finish,
                options_rs,
                options_rf,
                options_ls,
                options_lf
            ],
        )
        .mount("/", StaticFiles::from("static/dist").rank(-1))
}
