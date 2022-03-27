use crate::image;
use crate::image_downloader;
use crate::image_downloader_lxd;
use crate::user;

#[test]
#[ignore]
fn test_download_image() {
    let arch = user::CpuArchitecture::Amd64;
    let downloader = image_downloader_lxd::Downloader::new("alpine", "3.15", arch).unwrap();
    let user = user::User::new().unwrap();
    let image = image::Image::new("alpine", "3.15", user, downloader).unwrap();
    image.download_image().unwrap();
}

#[test]
#[ignore]
fn test_check_rootfs_newest() {
    let arch = user::CpuArchitecture::Amd64;
    let downloader = image_downloader_lxd::Downloader::new("alpine", "3.15", arch).unwrap();
    let user = user::User::new().unwrap();
    let image = image::Image::new("alpine", "3.15", user, downloader).unwrap();
    image.check_rootfs_newest();
}
