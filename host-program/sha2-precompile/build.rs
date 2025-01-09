fn main() {
    let pre_guest_path = format!(
        "{}/../../guest-program/sha2-rust",
        env!("CARGO_MANIFEST_DIR")
    );
    zkm_build::build_program(&pre_guest_path);
    let pre_guest_target_path = format!(
        "{}/{}/{}",
        pre_guest_path,
        zkm_build::DEFAULT_OUTPUT_DIR,
        zkm_build::BUILD_TARGET
    );
    println!(
        "cargo:rustc-env=PRE_GUEST_TARGET_PATH={}",
        pre_guest_target_path
    );

    let guest_path = format!(
        "{}/../../guest-program/sha2-precompile",
        env!("CARGO_MANIFEST_DIR")
    );
    zkm_build::build_program(&guest_path);
    let guest_target_path = format!(
        "{}/{}/{}",
        guest_path,
        zkm_build::DEFAULT_OUTPUT_DIR,
        zkm_build::BUILD_TARGET
    );
    println!("cargo:rustc-env=GUEST_TARGET_PATH={}", guest_target_path);
}
