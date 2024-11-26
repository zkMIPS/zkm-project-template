extern crate libc;
use libc::c_int;
use std::os::raw::c_char;

extern "C" {
    fn Stark2Snark(inputdir: *const c_char, outputdir: *const c_char) -> c_int;
    fn Setup(inputdir: *const c_char) -> c_int;
}

#[cfg(feature = "snark")]
pub fn prove_snark(inputdir: &str, outputdir: &str) -> anyhow::Result<bool> {
    let inputdir = std::ffi::CString::new(inputdir).unwrap();
    let outputdir = std::ffi::CString::new(outputdir).unwrap();

    let ret = unsafe { Stark2Snark(inputdir.as_ptr(), outputdir.as_ptr()) };
    if ret == 0 {
        Ok(true)
    } else {
        Ok(false)
    }
}

#[cfg(not(feature = "snark"))]
pub fn prove_snark(inputdir: &str, outputdir: &str) -> anyhow::Result<bool> {
    panic!("not support snark");
}

#[cfg(feature = "snark")]
pub fn setup(inputdir: &str) -> anyhow::Result<bool> {
    let inputdir = std::ffi::CString::new(inputdir).unwrap();

    let ret = unsafe { Setup(inputdir.as_ptr()) };
    if ret == 0 {
        Ok(true)
    } else {
        Ok(false)
    }
}

#[cfg(not(feature = "snark"))]
pub fn setup(inputdir: &str) -> anyhow::Result<bool> {
    panic!("not support setup");
}
