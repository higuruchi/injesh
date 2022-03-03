use std::path::Path;

/// rootfsを提供するサーバへアクセスするためのトレイト
pub trait Downloader {
    /// rootfsを提供しているサーバからアーカイブファイルをダウンロードする
    fn download_rootfs(&self, to: &Path) -> Result<(), Box<dyn std::error::Error>>;

    /// rootfsのアーカイブファイルのhashファイルをダウンロードする
    fn download_rootfs_hash(&self, to: &Path) -> Result<(), Box<dyn std::error::Error>>;

    /// ローカルにあるrootfsが最新のものかどうかをチェックする
    fn check_rootfs_newest(
        &self,
        local_rootfs_hash_path: &Path,
    ) -> Result<bool, Box<dyn std::error::Error>>;
}
