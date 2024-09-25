use common::file;
use std::fs;
use std::env;
use std::path::Path;
use std::time::Instant;

use zkm_sdk::{prover::ProverInput, ProverClient};

use std::fs::read;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::try_init().unwrap_or_default();
    log::info!("new prover client.");
    let prover_client = ProverClient::new().await; //ENV: ZKM_PROVER=local
    log::info!("new prover client,ok.");

    let seg_size = env::var("SEG_SIZE").unwrap_or("131072".to_string());
    let seg_size2 = seg_size.parse::<_>().unwrap_or(131072);
    let execute_only = env::var("EXECUTE_ONLY").unwrap_or("false".to_string());
    let execute_only2 = execute_only.parse::<bool>().unwrap_or(false);
    let elf_path = env::var("ELF_PATH")
        .unwrap_or("guest-program/mips-elf/zkm-mips-elf-revme-rust".to_string());
    let public_input_path = env::var("PUBLIC_INPUT_PATH").unwrap_or("".to_string());
    let private_input_path = env::var("PRIVATE_INPUT_PATH").unwrap_or("".to_string());
    let input = ProverInput {
        elf: read(elf_path).unwrap(),
        public_inputstream: read(public_input_path).unwrap_or("".into()),
        private_inputstream: read(private_input_path).unwrap_or("".into()),
        seg_size: seg_size2,
        execute_only: execute_only2,
    };
    //
    let start = Instant::now();
    let output_dir = env::var("OUTPUT_DIR").unwrap_or("/tmp/zkm".to_string());
    match fs::create_dir_all(&output_dir) {
        Ok(_) => log::info!("{} created successfully.", &output_dir),
        Err(e) => {
            log::info!("Failed to create directory {}, err: {}", &output_dir, e);
            return Ok(());
        }
    }
    let proving_result = prover_client.prover.prove(&input, None).await;
    //match proverClient.await.prover.prover(&input,None).await {
    match proving_result {
        Ok(Some(prover_result)) => {
            let output_path = Path::new(&output_dir);
            let proof_result_path = output_path.join("snark_proof_with_public_inputs.json");
            let mut f = file::new(&proof_result_path.to_string_lossy());
            match  f.write(prover_result.proof_with_public_inputs.as_slice()) {
                Ok(bytes_written) => {
                    log::info!("Proof: successfully written {} bytes.", bytes_written);
                },
                Err(e) => {
                    log::info!("Proof: failed to write to file: {}", e);
                    return Ok(());
                }
            }
            //contract
            let output_path = Path::new(&output_dir);
            let contract_path = output_path.join("verifier.sol");
            let mut f = file::new(&contract_path.to_string_lossy());
            match  f.write(prover_result.solidity_verifier.as_slice()){
                Ok(bytes_written) => {
                    log::info!("Contract: successfully written {} bytes.", bytes_written);
                },
                Err(e) => {
                    log::info!("Contract: failed to write to file: {}", e);
                    return Ok(());
                }
            }
            log::info!("Generating proof successfully .The proof file and verifier contract are in the path {}.",&output_dir);
        }
        Ok(None) => {
            log::info!("Failed to generate proof.The result is None.");
        }
        Err(e) => {
            log::info!("Failed to generate proof. error: {}", e);
            return Ok(());
        }
    }

    let end = Instant::now();
    let elapsed = end.duration_since(start);
    log::info!("Elapsed time: {:?} secs", elapsed.as_secs());
    Ok(())
}
