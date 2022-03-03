use crate::{image_downloader, user};
use std::fs::{self, File};
use std::path::{Path, PathBuf};
use std::{error, fmt};
use tar::Archive;
use xz2::read::XzDecoder;

const ROOTFS_FILE: &str = "rootfs.tar.xz";
const ROOTFS_HASH_FILE: &str = "rootfs.tar.xz.asc";
const ROOTFS: &str = "rootfs";

#[derive(Debug)]
pub enum Error {
    ImageSyntaxError,
    ImageNotFound,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::ImageSyntaxError => write!(f, "Image Syntax Error"),
            Error::ImageNotFound => write!(f, "Image Not Found"),
        }
    }
}

impl error::Error for Error {}

/// rootfsを管理するための構造体
#[derive(Debug)]
pub struct Image<DO>
where
    DO: image_downloader::Downloader,
{
    /// ユーザが入力したrootfsのディストリビューションを格納する
    distribution: String,
    /// ユーザが入力したrootfsのバージョンを格納する
    version: String,
    ///　ユーザ依存の情報を格納する
    user: user::User,
    image_base_path: PathBuf,
    downloader: DO,
}

impl<DO> Image<DO>
where
    DO: image_downloader::Downloader,
{
    /// Image構造体のコンストラクタ
    ///
    /// # Example
    /// ```ignore
    /// use crate::user;
    /// use crate::image;
    /// use crate::image_downloader_lxd;
    ///
    /// let user = user::User::new().unwrap();
    /// let downloader = image_downloader_lxd::Downloader::new("ubuntu", "focal", "amd64");
    /// let image = image::Image::new("ubuntu", "focal", user, downloader);
    /// ```
    pub fn new(
        distribution: &str,
        version: &str,
        user: user::User,
        downloader: DO,
    ) -> Result<Image<DO>, Box<dyn std::error::Error>> {
        let image_base_path =
            PathBuf::from(&format!("{}/{}/{}", user.images(), distribution, version));

        Ok(Image {
            distribution: distribution.to_string(),
            version: version.to_string(),
            user: user,
            image_base_path: image_base_path,
            downloader: downloader,
        })
    }

    fn user(&self) -> &user::User {
        &self.user
    }

    fn distribution(&self) -> &str {
        &self.distribution
    }

    fn version(&self) -> &str {
        &self.version
    }

    /// 任意のディストリビューション、バージョンのrootfsやhash fileを入れるpathを返却
    ///
    /// # Example
    /// ```~/.injesh/images/alpine/3.15```
    fn image_base_path(&self) -> &Path {
        self.image_base_path.as_path()
    }

    /// 任意のディストリビューション、バージョンのhash fileへのpathを返却
    ///
    /// # Example
    /// ```~/.injesh/images/alpine/3.15/rootfs.tar.xz.asc```
    fn rootfs_hash_path(&self) -> PathBuf {
        PathBuf::from(&format!(
            "{}/{}",
            self.image_base_path().display(),
            ROOTFS_HASH_FILE
        ))
    }

    /// 任意のディストリビューション、バージョンのダウンロードされるrootfs.tar.xzへのpathを返却
    /// # Example
    /// ```~/.injesh/images/alpine/3.15/rootfs.tar.xz```
    fn downloaded_rootfs_path(&self) -> PathBuf {
        PathBuf::from(&format!(
            "{}/{}",
            self.image_base_path().display(),
            ROOTFS_FILE
        ))
    }

    /// 任意のディストリビューション、バージョンのrootfsへのpathを返却
    ///
    /// # Example
    /// ```~/.injesh/images/alpine/3.15/rootfs```
    fn rootfs_path(&self) -> PathBuf {
        PathBuf::from(&format!("{}/{}", self.image_base_path().display(), ROOTFS))
    }

    /// ローカルにrootfsイメージがあるかどうか調べる
    ///
    /// # Example
    /// ```ignore
    /// image.search_image();
    /// ```
    pub fn search_image(&self) -> Result<(), Box<dyn std::error::Error>> {
        if self.rootfs_hash_path().exists() {
            return Ok(());
        }
        Err(Error::ImageNotFound)?
    }

    /// ローカルにあるイメージが最新のものかどうかを調べる
    ///
    /// # Example
    /// ```ignore
    /// image.image_is_newes();
    /// ```
    pub fn check_rootfs_newest(&self) -> Result<bool, Box<dyn std::error::Error>> {
        self.downloader
            .check_rootfs_newest(&self.rootfs_hash_path())
    }

    /// rootfsイメージをダウンロードする
    /// 既にrootfsが存在する場合、削除してからダウンロードする
    /// そのため存在するかどうか、最新かどうかを確認してから
    /// 呼び出す必要がある
    ///
    /// # Example
    /// ```ignore
    /// image.download_image();
    /// ```
    pub fn download_image(&self) -> Result<(), Box<dyn std::error::Error>> {
        // ローカルにあるrootfsが最新のものかどうかを確認
        if Path::new(&self.rootfs_hash_path()).exists() {
            match self.check_rootfs_newest() {
                Ok(newest_flg) => match newest_flg {
                    true => return Ok(()),
                    false => { /* do nothing */ }
                },
                Err(_) => { /* do nothing */ }
            }
        }

        // rootfsディレクトリがないならば作る
        // ある場合はrootfsディレクトリ以下を削除する
        if !Path::new(&self.rootfs_path()).exists() {
            fs::create_dir_all(&self.rootfs_path())?;
        } else {
            fs::remove_dir_all(&self.rootfs_path())?;
        }

        self.downloader
            .download_rootfs(&self.downloaded_rootfs_path())?;

        self.downloader
            .download_rootfs_hash(&self.rootfs_hash_path())?;

        // ダウンロードしたrootfsを解凍
        let tar_xz = File::open(self.downloaded_rootfs_path())?;
        let tar = XzDecoder::new(tar_xz);
        let mut archive = Archive::new(tar);
        archive.unpack(self.rootfs_path())?;

        // ダウンロードしたtarファイルを削除
        fs::remove_file(self.downloaded_rootfs_path())?;

        Ok(())
    }
}
