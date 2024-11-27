extern crate libc;
use libc::c_int;
use std::os::raw::c_char;
use std::path::Path;

extern "C" {
    fn Stark2Snark(
        keypath: *const c_char,
        inputdir: *const c_char,
        outputdir: *const c_char,
    ) -> c_int;
    fn SetupAndGenerateSolVerifier(inputdir: *const c_char) -> c_int;
}

#[cfg(feature = "snark")]
pub fn prove_snark(keypath: &str, inputdir: &str, outputdir: &str) -> anyhow::Result<bool> {
    let path = Path::new(keypath);
    let pk_file = path.join("proving.key");
    let vk_file = path.join("verifying.key");

    if !pk_file.exists() || !vk_file.exists() {
        panic!(
            "The vk or pk doesn't exist in the path: {}. Please first set the SETUP_FLAG=true to run setup_and_generate_sol_verifier.",inputdir
        );
    }

    let keypath = std::ffi::CString::new(keypath).unwrap();
    let inputdir = std::ffi::CString::new(inputdir).unwrap();
    let outputdir = std::ffi::CString::new(outputdir).unwrap();

    let ret = unsafe { Stark2Snark(keypath.as_ptr(), inputdir.as_ptr(), outputdir.as_ptr()) };
    if ret == 0 {
        Ok(true)
    } else {
        Ok(false)
    }
}

#[cfg(not(feature = "snark"))]
pub fn prove_snark(keypath: &str, inputdir: &str, outputdir: &str) -> anyhow::Result<bool> {
    panic!("not support snark");
}

#[cfg(feature = "snark")]
pub fn setup_and_generate_sol_verifier(inputdir: &str) -> anyhow::Result<bool> {
    let inputdir = std::ffi::CString::new(inputdir).unwrap();

    let ret = unsafe { SetupAndGenerateSolVerifier(inputdir.as_ptr()) };
    if ret == 0 {
        Ok(true)
    } else {
        Ok(false)
    }
}

#[cfg(not(feature = "snark"))]
pub fn setup_and_generate_sol_verifier(inputdir: &str) -> anyhow::Result<bool> {
    panic!("not support setup");
}
