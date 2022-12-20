use crate::models::User;

use crate::persistence;

pub fn get_user(email: &str) -> Result<User, String> {
    match persistence::find_user(email) {
        Ok(result) => match result {
            Some(user) => Ok(user),
            None => {
                println!("User not found: {}", email);
                Err(String::from("User not found!"))
            }
        },
        Err(err) => {
            println!("Error finding user: {:?}", err);
            Err(String::from("User not found!"))
        }
    }
}
