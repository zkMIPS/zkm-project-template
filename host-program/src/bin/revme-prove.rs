use common::file;

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
    let proving_result = prover_client.prover.prove(&input, None).await;
    //match proverClient.await.prover.prover(&input,None).await {
    match proving_result {
        Ok(Some(prover_result)) => {
            if execute_only2 == false {
                log::info!("Generating proof successfully .The proof file and verifier contract are in the path {}.",&output_dir);
                let output_path = Path::new(&output_dir);
                let proof_result_path = output_path.join("snark_proof_with_public_inputs.json");
                let _ = file::new(&proof_result_path.to_string_lossy())
                    .write(prover_result.proof_with_public_inputs.as_slice());
                //contract
                let output_path = Path::new(&output_dir);
                let contract_path = output_path.join("verifier.sol");
                let _ = file::new(&contract_path.to_string_lossy())
                    .write(prover_result.solidity_verifier.as_slice());
            } else {
                if prover_result.output_stream.len() == 0 {
                    log::info!("output_stream.len() is too short: {}",prover_result.output_stream.len());
                    return Ok(());
                }
                log::info!("Executing the guest program  successfully.");
                let ret_data = bincode::deserialize_from(prover_result.output_stream.as_slice())
                .expect("deserialization failed");
                log::info!("ret_data: {:?}", ret_data);
            }
            
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
