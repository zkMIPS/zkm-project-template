fn main() {
    zkm_build::build_program(&format!(
        "{}/../../guest-program/mem-alloc-vec",
        env!("CARGO_MANIFEST_DIR")
    ));
}
