use std::path::Path;

/// rootfsを提供するサーバへアクセスするためのトレイト
pub trait Downloader {
    /// rootfsを提供しているサーバからアーカイブファイルをダウンロードする
    fn download_rootfs(
        &self,
        distribution: &str,
        version: &str,
        arch: &str,
        to: &Path,
    ) -> Result<(), Box<dyn std::error::Error>>;

    /// rootfsのアーカイブファイルのhashファイルをダウンロードする
    fn download_rootfs_hash(
        &self,
        distribution: &str,
        version: &str,
        arch: &str,
        to: &Path,
    ) -> Result<(), Box<dyn std::error::Error>>;

    /// ローカルにあるrootfsが最新のものかどうかをチェックする
    fn check_rootfs_newest(&self, to: &Path) -> Result<bool, Box<dyn std::error::Error>>;
}
