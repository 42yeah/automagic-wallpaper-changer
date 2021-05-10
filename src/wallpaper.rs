use std::{env, io, process::Command};
use std::os::raw::c_void;
use std::ffi::OsStr;
use std::iter;

// Code taken from https://github.com/reujab/wallpaper.rs/blob/master/src/macos.rs

#[cfg(any(target_os = "windows"))]
use std::os::windows::ffi::OsStrExt;
#[cfg(any(target_os = "windows"))]
use winapi::um::winuser::SystemParametersInfoW;
#[cfg(any(target_os = "windows"))]
use winapi::um::winuser::SPIF_SENDCHANGE;
#[cfg(any(target_os = "windows"))]
use winapi::um::winuser::SPIF_UPDATEINIFILE;
#[cfg(any(target_os = "windows"))]
use winapi::um::winuser::SPI_SETDESKWALLPAPER;

#[cfg(any(target_os = "windows"))]
pub fn set_wallpaper(path: &str) -> Result<(), io::Error> {
    unsafe {
        let path = OsStr::new(path)
            .encode_wide()
            // append null byte
            .chain(iter::once(0))
            .collect::<Vec<u16>>();
        let successful = SystemParametersInfoW(
            SPI_SETDESKWALLPAPER,
            0,
            path.as_ptr() as *mut c_void,
            SPIF_UPDATEINIFILE | SPIF_SENDCHANGE,
        ) == 1;

        if successful {
            Ok(())
        } else {
            Err(io::Error::last_os_error().into())
        }
    }
}

#[cfg(any(target_os = "linux"))]
pub fn set_wallpaper(path: &str) -> Result<(), io::Error> {
    unimplemented!();
}

#[cfg(any(target_os = "macos"))]
pub fn set_wallpaper(path: &str) -> Result<(), io::Error> {
    let current_dir = env::current_dir()?;
    let current_dir = current_dir.as_path().to_str().unwrap();
    Command::new("osascript")
        .arg("-e")
        .arg(format!("tell application \"Finder\" to set desktop picture to POSIX file \"{}/{}\"", current_dir, path))
        .spawn()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore = "Your wallpaper WILL be changed"]
    fn test_set_wallpaper() {
        set_wallpaper("ferris.png").unwrap()
    }
}
