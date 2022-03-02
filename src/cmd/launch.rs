use crate::command;
use crate::image_downloader::Downloader;
use crate::launch::Launch;

pub struct LaunchStruct;

impl<DO> Launch<DO> for LaunchStruct
where
    DO: Downloader,
{
    fn launch(&self, launch: &command::Launch<DO>) -> Result<(), Box<dyn std::error::Error>> {
        println!("execute launch!");
        Ok(())
    }
}

impl LaunchStruct {
    pub fn new<DO>() -> impl Launch<DO>
    where
        DO: Downloader,
    {
        LaunchStruct
    }
}
