fn main() {
    zkm_build::build_program(&format!("{}/../../guest-program/revme", env!("CARGO_MANIFEST_DIR")));
}