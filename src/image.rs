use crate::user;
use std::fs::{self, File};
use std::io;
use std::path::Path;
use std::{error, fmt};
use tar::Archive;
use xz2::read::XzDecoder;

const ROOTFS_SERVER_DOMAIN: &str = "https://us.lxd.images.canonical.com";
const ROOTFS_FILE: &str = "rootfs.tar.xz";
const ROOTFS_HASH_FILE: &str = "rootfs.tar.xz.asc";
const IMAGE_META: &str = "https://uk.lxd.images.canonical.com/meta/1.0/index-user";

/// rootfsを管理するための構造体
#[derive(Debug)]
pub struct Image {
    /// ユーザが入力したrootfsのディストリビューションを格納する
    distribution: String,
    /// ユーザが入力したrootfsのバージョンを格納する
    version: String,
    ///　ユーザ依存の情報を格納する
    user: user::User,
    /// 現在のローカルにあるrootfsが最新かどうかを示すフラグ
    newest: Option<bool>,
    /// ```https://us.lxd.images.canonical.com```から取得した
    /// rootfsのメタデータ郡を ユーザが入力したディストリビューション、バージョンを用いてフィルタリングし、
    /// メタデータを格納する
    images_meta: Vec<ImageMeta>,
}

#[derive(Debug)]
pub enum Error {
    ImageSyntaxError,
    ImageNotFound,
    ImageMetaNotFound,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::ImageSyntaxError => write!(f, "Image Syntax Error"),
            Error::ImageNotFound => write!(f, "Image Not Found"),
            Error::ImageMetaNotFound => write!(f, "Image Meta Not Found"),
        }
    }
}

impl error::Error for Error {}

impl Image {
    /// Image構造体のコンストラクタ
    ///
    /// # Example
    /// ```
    /// use crate::user;
    /// use crate::image;
    ///
    /// let user = user::User::new().unwrap();
    /// let image = image::Image::new("ubuntu/focal", user);
    /// ```
    pub fn new(image: &str, user: user::User) -> Result<Image, Box<dyn std::error::Error>> {
        let distri_and_version: Vec<&str> = image.split("/").collect();

        if distri_and_version.len() != 2 {
            Err(Error::ImageSyntaxError)?
        }

        let images_meta = ImageMeta::new(&user, distri_and_version[0], distri_and_version[1])?;

        let distribution = distri_and_version[0].to_string();
        let version = distri_and_version[1].to_string();

        Ok(Image {
            distribution: distribution,
            version: version,
            user: user,
            newest: None,
            images_meta: images_meta,
        })
    }

    pub fn user(&self) -> &user::User {
        &self.user
    }

    pub fn distribution(&self) -> &str {
        &self.distribution
    }

    pub fn version(&self) -> &str {
        &self.version
    }

    pub fn images_meta(&self) -> &Vec<ImageMeta> {
        &self.images_meta
    }

    /// ローカルにrootfsイメージがあるかどうか調べる
    ///
    /// # Example
    /// ```
    /// image.search_image();
    /// ```
    pub fn search_image(&self) -> Result<(), Box<dyn std::error::Error>> {
        if Path::new(&format!(
            "{}/{}/{}",
            self.user().images(),
            self.distribution(),
            self.version()
        ))
        .exists()
        {
            return Ok(());
        }
        Err(Error::ImageNotFound)?
    }

    pub fn image_is_newest(&self) -> Result<bool, Box<dyn std::error::Error>> {
        let path = self.newest_url().ok_or(Error::ImageMetaNotFound)?;

        let downloaded_rootfs_hash = reqwest::blocking::get(format!(
            "{}/{}/{}",
            ROOTFS_SERVER_DOMAIN, path, ROOTFS_HASH_FILE
        ))?
        .text()?;

        let rootfs_hash = fs::read_to_string(format!(
            "{}/{}/{}/{}",
            self.user().images(),
            self.distribution(),
            self.version(),
            ROOTFS_HASH_FILE
        ))?;

        if downloaded_rootfs_hash == rootfs_hash {
            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// rootfsイメージをダウンロードする
    /// 既にrootfsが存在する場合、削除してからダウンロードする
    /// そのため存在するかどうか、最新かどうかを確認してから
    /// 呼び出す必要がある
    ///
    /// # Example
    /// ```
    /// image.download_image();
    /// ```
    pub fn download_image(&self) -> Result<(), Box<dyn std::error::Error>> {
        self.setup_rootfs_directory()?;

        let src = self.newest_url().ok_or(Error::ImageMetaNotFound)?;

        let rootfs_filenmae = format!(
            "{}/{}/{}/{}",
            self.user().images(),
            self.distribution(),
            self.version(),
            ROOTFS_FILE
        );
        let hash_filename = format!(
            "{}/{}/{}/{}",
            self.user().images(),
            self.distribution(),
            self.version(),
            ROOTFS_HASH_FILE
        );

        let rootfs_resp =
            reqwest::blocking::get(format!("{}/{}/{}", ROOTFS_SERVER_DOMAIN, src, ROOTFS_FILE))?
                .bytes()?;
        let mut rootfs_out = File::create(&rootfs_filenmae)?;
        io::copy(&mut rootfs_resp.as_ref(), &mut rootfs_out)?;

        let hash_resp = reqwest::blocking::get(format!(
            "{}/{}/{}",
            ROOTFS_SERVER_DOMAIN, src, ROOTFS_HASH_FILE
        ))?
        .bytes()?;
        let mut hash_out = File::create(&hash_filename)?;
        io::copy(&mut hash_resp.as_ref(), &mut hash_out)?;

        let tar_xz = File::open(&rootfs_filenmae)?;
        let tar = XzDecoder::new(tar_xz);
        let mut archive = Archive::new(tar);
        archive.unpack(format!(
            "{}/{}/{}/{}",
            self.user().images(),
            self.distribution(),
            self.version(),
            "rootfs"
        ))?;

        fs::remove_file(rootfs_filenmae)?;

        Ok(())
    }

    /// rootfsイメージを格納するディレクトリを生成する
    ///
    /// # Example
    /// ```
    /// image.setup_rootfs_directory();
    /// ```
    fn setup_rootfs_directory(&self) -> Result<(), Box<dyn std::error::Error>> {
        // rootfsのディストリビューションを表すディレクトリのチェック
        if !Path::new(&format!("{}/{}", self.user().images(), self.distribution())).exists() {
            fs::create_dir(format!("{}/{}", self.user().images(), self.distribution()))?
        }

        // rootfsのバージョンを表すディレクトリのチェック
        if !Path::new(&format!(
            "{}/{}/{}",
            self.user().images(),
            self.distribution(),
            self.version()
        ))
        .exists()
        {
            fs::create_dir(format!(
                "{}/{}/{}",
                self.user().images(),
                self.distribution(),
                self.version()
            ))?
        }

        if !Path::new(&format!(
            "{}/{}/{}/{}",
            self.user().images(),
            self.distribution(),
            self.version(),
            "rootfs"
        ))
        .exists()
        {
            fs::create_dir(format!(
                "{}/{}/{}/{}",
                self.user().images(),
                self.distribution(),
                self.version(),
                "rootfs"
            ))?;
        } else {
            fs::remove_dir_all(format!(
                "{}/{}/{}/{}",
                self.user().images(),
                self.distribution(),
                self.version(),
                "rootfs"
            ))?;
        }

        Ok(())
    }

    /// 最新のrootfsのurlを取得する
    ///
    /// # Example
    /// ```
    /// let url = image.newest_url();
    /// ```
    ///
    /// # Response Example
    /// /images/ubuntu/focal/ppc64el/default/20220227_07:42/
    fn newest_url(&self) -> Option<&str> {
        let mut newest_image_url = None;
        let mut newest_time = Time::new(0, 0, 0);

        // self.images_meta().iter()
        for meta in self.images_meta().iter() {
            if meta.time().compare(&newest_time) == 1 {
                newest_image_url = Some(meta.path());

                newest_time.date = meta.time().date;
                newest_time.hour = meta.time().hour;
                newest_time.minutes = meta.time().minutes;
            }
        }
        newest_image_url
    }
}

/// rootfsイメージのメタデータを管理する構造体
#[derive(Debug)]
pub struct ImageMeta {
    /// ディストリビューション名
    distribution: String,
    /// バージョン
    version: String,
    /// CPUアーキテクチャ
    arch: String,
    /// rootfsイメージがアップロードされた時間
    time: Time,
    /// rootfsイメージへのURLの一部
    path: String,
}

impl ImageMeta {
    /// ImageMeta構造体のコンストラクタ
    ///
    /// # Exapmle
    /// ```
    /// let image = ImageMeta::new(user, "ubuntu", "focal");
    /// ```
    fn new(
        user: &user::User,
        distri: &str,
        version: &str,
    ) -> Result<Vec<ImageMeta>, Box<dyn std::error::Error>> {
        let resp = reqwest::blocking::get(IMAGE_META)?.text()?;
        let image_info: Vec<&str> = resp.split('\n').collect();
        let image_candidates: Vec<ImageMeta> = image_info
            .into_iter()
            .filter_map(|image_candidate| {
                let image_parsed_info: Vec<&str> = image_candidate.split(';').collect();

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
                        time: parse_time(image_parsed_info[4]),
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

    fn time(&self) -> &Time {
        &self.time
    }

    fn path(&self) -> &str {
        &self.path
    }
}

/// rootfsのアップデートされた時間を管理する構造体
#[derive(Debug)]
pub struct Time {
    /// 年、月、日を格納
    ///
    /// # Example
    /// 2022年2月27日の場合
    /// 20220227
    date: u64,
    /// 時間を格納する
    hour: u64,
    /// 分を格納する
    minutes: u64,
}

impl Time {
    /// Time構造体のコンストラクタ
    ///
    /// # Example
    /// ```
    /// let time = Time::new(20220227, 2 27)
    /// ```
    fn new(date: u64, hour: u64, minutes: u64) -> Time {
        Time {
            date: date,
            hour: hour,
            minutes: minutes,
        }
    }

    /// Time構造体を比較する
    ///
    /// # Example
    /// ```
    /// time1.compare(time1);
    /// ```
    ///
    /// # Response
    /// time1の方が大きい場合、1を返却する
    /// time2の方が大きい場合、-1を返却する
    /// time1とtime2が同じの場合0を返却する
    fn compare(&self, time: &Time) -> i64 {
        let time1 = self.date * 10000 + self.hour * 100 + self.minutes;
        let time2 = time.date * 10000 + time.hour * 100 + time.minutes;

        if time1 > time2 {
            1
        } else if time1 == time2 {
            0
        } else {
            -1
        }
    }
}

fn parse_time(time_str: &str) -> Time {
    let date: Vec<&str> = time_str.split('_').collect();
    let time: Vec<&str> = date[1].split(':').collect();

    Time {
        date: date[0].parse::<u64>().unwrap(),
        hour: time[0].parse::<u64>().unwrap(),
        minutes: time[1].parse::<u64>().unwrap(),
    }
}

mod tests {
    use super::*;

    #[test]
    fn test_compare() {
        let time1_data = vec![
            Time::new(20220227, 2, 27),
            Time::new(20220227, 2, 28),
            Time::new(20220227, 2, 28),
        ];

        let time2_data = vec![
            Time::new(20220227, 2, 28),
            Time::new(20220227, 2, 27),
            Time::new(20220227, 2, 28),
        ];
        let ans_data = vec![-1, 1, 0];

        for ((time1, time2), ans) in time1_data
            .iter()
            .zip(time2_data.iter())
            .zip(ans_data.iter())
        {
            assert_eq!(time1.compare(time2), *ans);
        }
    }
}
