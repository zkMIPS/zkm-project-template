use anyhow::bail;
use anyhow::Result;
use std::env;
use std::fs::read;
use std::time::Instant;
use zkm_sdk::{prover::ClientCfg, prover::ProverInput, ProverClient};

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::try_init().unwrap_or_default();

    let elf_path = std::env::var("ELF_PATH").unwrap_or(env!("GUEST_TARGET_PATH").to_string());
    std::env::set_var("ELF_PATH", elf_path);
    let json_path = env::var("JSON_PATH").expect("JSON PATH is missing");
    env::set_var("ARGS", json_path);

    let (client_config, prover_input) = ClientCfg::from_env(set_guest_input);

    // THIS IS ARGS
    let prover_client = ProverClient::new(&client_config).await;
    log::info!("new prover client,ok.");

    //excuting the setup_and_generate_sol_verifier
    if prover_input.snark_setup {
        match prover_client
            .setup_and_generate_sol_verifier(
                &client_config.zkm_prover_type,
                &client_config.vk_path,
                &prover_input,
            )
            .await
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
                //the revme guest program doesn't have outputs messages.
                prover_client
                    .print_guest_execution_output(false, &prover_result)
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

fn set_guest_input(input: &mut ProverInput, args: Option<&str>) {
    input.public_inputstream = read(args.expect("args false")).unwrap(); //the json file has been bincoded.
}
