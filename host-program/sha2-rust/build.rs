fn main() {
    zkm_build::build_program(&format!(
        "{}/../../guest-program/sha2-rust",
        env!("CARGO_MANIFEST_DIR")
    ));
}
