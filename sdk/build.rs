fn main() -> Result<(), Box<dyn std::error::Error>> {
    if let Some(_snark_flag) = std::env::var_os("NO_USE_SNARK") {
        tonic_build::configure()
            .protoc_arg("--experimental_allow_proto3_optional")
            .compile(&["src/proto/stage.proto"], &["src/proto"])?;
    } else {
        println!("cargo:rustc-link-search=native=./sdk/src/local/libsnark");
        println!("cargo:rustc-link-lib=dylib=snark");
        tonic_build::configure()
            .protoc_arg("--experimental_allow_proto3_optional")
            .compile(&["src/proto/stage.proto"], &["src/proto"])?;
    }

    Ok(())
}
