use crate::image_downloader::Downloader;
use crate::{command, setting};

pub trait Launch<DO>
where
    DO: Downloader,
{
    fn launch(&self, launch: &mut command::Launch<DO>) -> Result<(), Box<dyn std::error::Error>>;
}
