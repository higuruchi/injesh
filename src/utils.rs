//! This module conatins common utility functions.
//!
//! - check initialized
//! - getting PID from docker container name or id
//! - generating rootfs from image server

pub fn check_initialized() -> Result<(), Box<dyn std::error::Error>> {
    let user_info = crate::user::User::new()?;
    if !std::path::Path::new(user_info.injesh_home()).exists()
        || !std::path::Path::new(user_info.images()).exists()
        || !std::path::Path::new(user_info.containers()).exists()
    {
        Err(crate::command::Error::NotInitialized)?
    }

    Ok(())
}

// mod tests {
//     use super::*;

//     #[test]
//     fn test_check_initialized() {
//         assert!(check_initialized().is_err());

//         // mkdir
//         std::fs::create_dir_all(crate::user::User::new().unwrap().images());
//         std::fs::create_dir_all(crate::user::User::new().unwrap().containers());

//         assert!(check_initialized().is_ok());

//         // clean up
//         std::fs::remove_dir_all(crate::user::User::new().unwrap().injesh_home());
//     }
// }
