use crate::image_downloader;
use chrono::NaiveDateTime;
use regex::Regex;
use std::cmp::Ordering;
use std::fs::{self, File};
use std::io;
use std::path::Path;
use std::{error, fmt};

const ROOTFS_SERVER_DOMAIN: &str = "https://us.lxd.images.canonical.com";
const ROOTFS_FILE: &str = "rootfs.tar.xz";
const ROOTFS_HASH_FILE: &str = "rootfs.tar.xz.asc";
const IMAGE_META_URL: &str = "https://uk.lxd.images.canonical.com/meta/1.0/index-user";
const ROOTFS: &str = "rootfs";

#[derive(Debug)]
pub enum Error {
    ImageNotFound,
    ImageMetaNotFound,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::ImageNotFound => write!(f, "Image Not Found"),
            Error::ImageMetaNotFound => write!(f, "Image Meta Not Found"),
        }
    }
}

impl error::Error for Error {}

pub struct Downloader {
    /// ```https://us.lxd.images.canonical.com```から取得した
    /// rootfsのメタデータ郡を ユーザが入力したディストリビューション、バージョンを用いてフィルタリングし、
    /// メタデータを格納する
    ///
    /// # Exampld
    /// ```ignore
    /// [
    ///     ImageMeta{
    ///         distribution: "ubuntu",
    ///         version: "focal",
    ///         atch: "amd64",
    ///         time: NaviveDateTime,
    ///         path: "/images/ubuntu/focal/arm64/default/20220227_07:43/",
    ///     },
    ///     ImageMeta{
    ///         distribution: "ubuntu",
    ///         version: "focal",
    ///         atch: "amd64",
    ///         time: NaviveDateTime,
    ///         path: "/images/ubuntu/focal/arm64/default/20220228_07:43/",
    ///     },
    /// ]
    /// ```
    specific_images_meta: Vec<ImageMeta>,
}

impl Downloader {
    /// Downloader構造体のコンストラクタ
    ///
    /// 初期化すると同時にhttps://uk.lxd.images.canonical.com/meta/1.0/index-userから
    /// メタデータをダウンロードしてくる
    ///
    /// # Example
    /// ```ignore
    /// let image_downloader_lxd = Downloader::new("alpine", "3.15", "arm64").unwrap();
    /// ```
    pub fn new(
        distribution: &str,
        version: &str,
        arch: &str,
    ) -> Result<impl image_downloader::Downloader, Box<dyn std::error::Error>> {
        let image_meta = ImageMeta::new(distribution, version, arch)?;

        Ok(Downloader {
            specific_images_meta: image_meta,
        })
    }

    /// ImageMeta構造体のベクタの参照を返却
    fn specific_images_meta(&self) -> &Vec<ImageMeta> {
        &self.specific_images_meta
    }

    /// 最新のrootfsのurlを取得する
    ///
    /// # Example
    /// ```ignore
    /// let url = image.newest_url();
    /// ```
    ///
    /// # Response Example
    /// /images/ubuntu/focal/ppc64el/default/20220227_07:42/
    fn newest_url(&self) -> Option<&str> {
        let mut newest_image_url = None;
        let mut newest_time = NaiveDateTime::from_timestamp(0, 0);

        for meta in self.specific_images_meta().iter() {
            if meta.time().cmp(&newest_time) == Ordering::Greater {
                newest_image_url = Some(meta.path());
                newest_time = meta.time().clone();
            }
        }
        newest_image_url
    }
}

impl image_downloader::Downloader for Downloader {
    fn download_rootfs(&self, destination: &Path) -> Result<(), Box<dyn std::error::Error>> {
        let newest_path = self.newest_url().ok_or(Error::ImageMetaNotFound)?;

        let rootfs_resp = reqwest::blocking::get(format!(
            "{}/{}/{}",
            ROOTFS_SERVER_DOMAIN, newest_path, ROOTFS_FILE
        ))?
        .bytes()?;
        // ダウンロードしたrootfsデータを書き込むファイルを作成 and 書き込み
        let mut rootfs_out = File::create(destination)?;
        io::copy(&mut rootfs_resp.as_ref(), &mut rootfs_out)?;

        Ok(())
    }

    fn download_rootfs_hash(&self, destination: &Path) -> Result<(), Box<dyn std::error::Error>> {
        let newest_path = self.newest_url().ok_or(Error::ImageMetaNotFound)?;

        // rootfsのhashフィあるをダウンロード
        let hash_resp = reqwest::blocking::get(format!(
            "{}/{}/{}",
            ROOTFS_SERVER_DOMAIN, newest_path, ROOTFS_HASH_FILE
        ))?
        .bytes()?;

        let mut hash_out = File::create(destination)?;
        io::copy(&mut hash_resp.as_ref(), &mut hash_out)?;

        Ok(())
    }

    fn check_rootfs_newest(
        &self,
        local_rootfs_hash_path: &Path,
    ) -> Result<bool, Box<dyn std::error::Error>> {
        let newest_path = self.newest_url().ok_or(Error::ImageMetaNotFound)?;

        let downloaded_rootfs_hash = reqwest::blocking::get(format!(
            "{}/{}/{}",
            ROOTFS_SERVER_DOMAIN, newest_path, ROOTFS_HASH_FILE
        ))?
        .text()?;

        let rootfs_hash = match fs::read_to_string(local_rootfs_hash_path) {
            Ok(rootfs_hash) => rootfs_hash,
            Err(e) => {
                return Ok(false);
            }
        };

        if downloaded_rootfs_hash == rootfs_hash {
            Ok(true)
        } else {
            Ok(false)
        }
    }
}

/// rootfsイメージのメタデータを管理する構造体
#[derive(Debug)]
struct ImageMeta {
    /// ディストリビューション名
    distribution: String,
    /// バージョン
    version: String,
    /// CPUアーキテクチャ
    arch: String,
    /// rootfsイメージがアップロードされた時間
    time: NaiveDateTime,
    /// rootfsイメージへのURLの一部
    path: String,
}

impl ImageMeta {
    /// ImageMeta構造体のコンストラクタ
    ///
    /// # Exapmle
    /// ```ignore
    /// let image = ImageMeta::new(user, "ubuntu", "focal");
    /// ```
    fn new(
        distri: &str,
        version: &str,
        arch: &str,
    ) -> Result<Vec<ImageMeta>, Box<dyn std::error::Error>> {
        let resp = reqwest::blocking::get(IMAGE_META_URL)?.text()?;
        let image_info: Vec<&str> = resp.split('\n').collect();
        let re = Regex::new(r"^(.+;){5}(/.+){6}/$")?;

        let image_candidates: Vec<ImageMeta> = image_info
            .into_iter()
            .filter_map(|image_candidate| {
                if !re.is_match(image_candidate) {
                    return None;
                }

                let image_parsed_info: Vec<&str> = image_candidate.split(';').collect();

                let time = match parse_time(image_parsed_info[4]) {
                    Ok(time) => time,
                    Err(_) => return None,
                };

                if image_parsed_info[0] == distri &&
                image_parsed_info[1] == version &&
                // TODO: ユーザのアーキテクチャから分岐
                image_parsed_info[2] == "amd64" &&
                image_parsed_info[3] == "default"
                {
                    Some(ImageMeta {
                        distribution: image_parsed_info[0].to_string(),
                        version: image_parsed_info[1].to_string(),
                        arch: image_parsed_info[2].to_string(),
                        time: time,
                        path: image_parsed_info[5].to_string(),
                    })
                } else {
                    None
                }
            })
            .collect();

        Ok(image_candidates)
    }

    fn distribution(&self) -> &str {
        &self.distribution
    }

    fn arch(&self) -> &str {
        &self.arch
    }

    fn time(&self) -> &NaiveDateTime {
        &self.time
    }

    fn path(&self) -> &str {
        &self.path
    }
}

fn parse_time(time_str: &str) -> Result<NaiveDateTime, Box<dyn std::error::Error>> {
    let date = NaiveDateTime::parse_from_str(time_str, "%Y%m%d_%H:%M")?;
    Ok(date)
}

mod tests {
    use super::*;

    #[test]
    #[ignore]
    fn test_download_rootfs() {
        use crate::image_downloader::Downloader as DownloaderTrait;

        let image_downloader_lxd = Downloader::new("alpine", "3.15", "arm64").unwrap();
        image_downloader_lxd
            .download_rootfs(Path::new("/tmp/rootfs.tar.xz"))
            .unwrap();

        assert!(Path::new("/tmp/rootfs.tar.xz").exists())
    }

    #[test]
    #[ignore]
    fn download_rootfs_hash() {
        use crate::image_downloader::Downloader as DownloaderTrait;

        let image_downloader_lxd = Downloader::new("alpine", "3.15", "arm64").unwrap();
        image_downloader_lxd
            .download_rootfs_hash(Path::new("/tmp/rootfs.tar.xz.asc"))
            .unwrap();

        assert!(Path::new("/tmp/rootfs.tar.xz.asc").exists())
    }

    #[test]
    #[ignore]
    fn test_check_rootfs_newest() {
        use crate::image_downloader::Downloader as DownloaderTrait;

        let image_downloader_lxd = Downloader::new("alpine", "3.15", "arm64").unwrap();
        image_downloader_lxd
            .check_rootfs_newest(Path::new("/tmp/rootfs.tar.xz.asc"))
            .unwrap();

        let result = image_downloader_lxd
            .check_rootfs_newest(Path::new("/tmp/rootfs.tar.xz.asc"))
            .unwrap();
        assert!(result);
    }
}
