use anyhow::bail;
use anyhow::Result;
use std::time::Instant;
use zkm_sdk::{prover::ClientCfg, prover::ProverInput, ProverClient};

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::try_init().unwrap_or_default();
    let elf_path = std::env::var("ELF_PATH").unwrap_or(env!("GUEST_TARGET_PATH").to_string());
    std::env::set_var("ELF_PATH", elf_path);

    let set_guest_input = |_: &mut ProverInput, _: Option<&str>| {};
    let (client_config, prover_input) = ClientCfg::from_env(set_guest_input);
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
                    .expect("Process proof results false");
            } else {
                //only excute the guest program without generating the proof.
                //the mem-alloc-vec guest program doesn't have output messages.
                prover_client
                    .print_guest_execution_output(false, &prover_result)
                    .expect("Print guest program excution's output false.");
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
