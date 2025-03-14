use anyhow::bail;
use anyhow::Result;
use sha2::{Digest, Sha256};
use std::time::Instant;
use zkm_sdk::{prover::ClientCfg, prover::ProverInput, ProverClient};

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::try_init().unwrap_or_default();
    let elf_path = std::env::var("ELF_PATH").unwrap_or(env!("GUEST_TARGET_PATH").to_string());
    std::env::set_var("ELF_PATH", elf_path);

    let (client_config, prover_input) = ClientCfg::from_env(set_guest_input);
    log::info!("new prover client:");
    let prover_client = ProverClient::new(&client_config).await;
    log::info!("new prover client,ok.");

    //excuting the setup_and_generate_sol_verifier
    if prover_input.snark_setup {
        match zkm_recursion::groth16_setup(&client_config.vk_path)
        {
            Ok(()) => log::info!("Succussfully setup_and_generate_sol_verifier."),
            Err(e) => {
                log::info!("Error during setup_and_generate_sol_verifier: {}", e);
                bail!("Failed to setup_and_generate_sol_verifier.");
            }
        }
    }

    let start = Instant::now();
    let proving_result = prover_client.prover.prove(&prover_input, None).await;
    match proving_result {
        Ok(Some(prover_result)) => {
            if !prover_input.execute_only {
                //excute the guest program and generate the proof
                prover_client
                    .process_proof_results(
                        &prover_result,
                        &prover_input,
                        &client_config.zkm_prover_type,
                    )
                    .expect("process proof results error");
            } else {
                //only excute the guest program without generating the proof.
                //the sha2-rust guest program has outputs messages, which are basic type.
                prover_client
                    .print_guest_execution_output(true, &prover_result)
                    .expect("print guest program excution's output false.");
            }
        }
        Ok(None) => {
            log::info!("Failed to generate proof.The result is None.");
            bail!("Failed to generate proof.");
        }
        Err(e) => {
            log::info!("Failed to generate proof. error: {}", e);
            bail!("Failed to generate proof.");
        }
    }

    let end = Instant::now();
    let elapsed = end.duration_since(start);
    log::info!("Elapsed time: {:?} secs", elapsed.as_secs());
    Ok(())
}

fn set_guest_input(input: &mut ProverInput, _param: Option<&str>) {
    let num_bytes: usize = 1024; //Notice! : if this value is small, it will not generate the  proof.
    let pri_input = vec![5u8; num_bytes];
    let mut hasher = Sha256::new();
    hasher.update(&pri_input);
    let result = hasher.finalize();
    let output: [u8; 32] = result.into();

    // assume the  arg[0] = hash(public input), and the arg[1] = public input.
    let public_input = output.to_vec();
    let mut pub_buf = Vec::new();
    bincode::serialize_into(&mut pub_buf, &public_input)
        .expect("public_input serialization failed");

    let mut pri_buf = Vec::new();
    bincode::serialize_into(&mut pri_buf, &pri_input).expect("private_input serialization failed");

    input.public_inputstream = pub_buf;
    input.private_inputstream = pri_buf;
}
