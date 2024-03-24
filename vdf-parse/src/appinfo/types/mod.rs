pub struct AppInfo2022 {
    app_id: u32,
    size: u32, /// until end of vdf
    info_state: u32, 
    last_updated: u32,
    pics_token: u64,
    sha1: [u8; 20], /// of appinfo
    change_number: u32,
    sha1: [u8; 20], // of binary vdf
}

pub struct AppInfo {
    app_id: u32,
    size: u32,
    info_state: u32,
    last_updated: u32,
    pics_token: u64,
    sha1: [u8; 20],
    change_number: u32,
}