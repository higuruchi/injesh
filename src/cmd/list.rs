use crate::command::{self, list_error::Error};
use crate::list::List;
use std::fs;

pub struct ListStruct;

impl List for ListStruct {
    fn list(&self, list: &command::List) -> Result<(), Box<dyn std::error::Error>> {
        let user_info = list.get_user();
        let container_names = extract_container_names(user_info)?;

        if container_names.is_empty() {
            Err(Error::NoContainers)?
        }

        println!("{}", container_names.trim());

        Ok(())
    }
}

fn extract_container_names(
    user_info: &crate::user::User,
) -> Result<String, Box<dyn std::error::Error>> {
    let mut container_names = String::new();

    match fs::read_dir(user_info.containers()) {
        Ok(paths) => {
            for path in paths.flatten() {
                if path.path().is_dir() {
                    let container_name = path
                        .file_name()
                        .into_string()
                        .map_err(|osstring| format!("{:?}", osstring))?;
                    container_names += &format!("{}\n", container_name);
                }
            }
        }
        Err(why) => Err(Error::ReadDirError(why))?,
    }

    Ok(container_names)
}

impl ListStruct {
    pub fn new() -> Self {
        ListStruct
    }
}

impl Default for ListStruct {
    fn default() -> Self {
        Self::new()
    }
}

mod tests {
    use super::*;

    #[test]
    fn test_extract_container_names() -> Result<(), Box<dyn std::error::Error>> {
        let u = command::List::new()?;
        let i = u.get_user();
        let container_names = extract_container_names(&i);
        assert_eq!(
            container_names
                .unwrap_err()
                .downcast::<Error>()?
                .to_string(),
            "Failed to reading /home/runner/.injesh: No such file or directory (os error 2)."
                .to_string()
        );
        // mkdir
        fs::create_dir_all(format!("{}/hoge", u.get_user().containers()))?;
        fs::create_dir_all(format!("{}/huga", u.get_user().containers()))?;
        let container_names = extract_container_names(&i);
        assert_eq!(container_names.unwrap(), "hoge\nhuga\n".to_string());

        // clean up
        fs::remove_dir_all(format!("{}", u.get_user().injesh_home()))?;

        Ok(())
    }
}
