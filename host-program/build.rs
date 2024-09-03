fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::compile_protos("proto/src/proto/prover/v1/prover.proto")?;
    tonic_build::compile_protos("proto/src/proto/stage/v1/stage.proto")?;
    Ok(())
}
