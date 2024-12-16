fn main() {
    let guest_path = format!(
        "{}/../../guest-program/mem-alloc-vec",
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
