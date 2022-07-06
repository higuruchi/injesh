use crate::{command, setting};
use crate::image_downloader::Downloader;

pub trait Launch<DO, RW>
where
    DO: Downloader,
    RW: setting::Reader + setting::Writer,
{
    fn launch(&self, launch: &mut command::Launch<DO, RW>) -> Result<(), Box<dyn std::error::Error>>;
}
