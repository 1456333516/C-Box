#[allow(dead_code)]
pub fn is_windows() -> bool {
    cfg!(target_os = "windows")
}
