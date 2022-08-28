use nix::unistd::{Gid, Uid};
use std::fs::OpenOptions;

pub fn new_uidmap(uid: &Uid) -> Result<(), Box<dyn std::error::Error>> {
    use std::ffi::CString;
    let mut uidmap_file = OpenOptions::new()
        .read(true)
        .write(true)
        .open("/proc/self/uid_map")?;
    let uidmap = CString::new(format!("0 {} 1", uid.as_raw()))?;

    use std::io::Write;
    uidmap_file.write(uidmap.as_bytes())?;

    Ok(())
}

pub fn new_gidmap(gid: &Gid) -> Result<(), Box<dyn std::error::Error>> {
    use std::ffi::CString;
    let mut setgroups_file = OpenOptions::new()
        .write(true)
        .open("/proc/self/setgroups")?;

    use std::io::Write;
    setgroups_file.write(b"deny")?;

    let mut gidmap_file = OpenOptions::new().write(true).open("/proc/self/gid_map")?;
    let gidmap = CString::new(format!("0 {} 1", gid.as_raw()))?;

    gidmap_file.write(gidmap.as_bytes())?;

    Ok(())
}
