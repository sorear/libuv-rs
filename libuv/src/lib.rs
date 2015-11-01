extern crate libuv_sys;
use std::ffi::CStr;

pub fn version_hex() -> u32 {
    unsafe { libuv_sys::uv_version() as u32 }
}

pub fn version_string() -> &'static str {
    unsafe { CStr::from_ptr(libuv_sys::uv_version_string()).to_str().unwrap() }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_version_hex() {
        assert_eq!(version_hex(), 0x10705);
    }

    #[test]
    fn test_version_string() {
        assert_eq!(version_string(), "1.7.5");
    }
}
