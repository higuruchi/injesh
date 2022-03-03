use crate::command;
use crate::image_downloader::Downloader;

pub trait Launch<DO>
where
    DO: Downloader,
{
    fn launch(&self, launch: &command::Launch<DO>) -> Result<(), Box<dyn std::error::Error>>;
}
