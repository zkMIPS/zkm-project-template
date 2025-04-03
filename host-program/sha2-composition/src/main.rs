use anyhow::bail;
use anyhow::Result;
use sha2::{Digest, Sha256};
use std::env;
use std::fs::read;
use std::time::Instant;
use zkm_sdk::{prover::ClientCfg, prover::ProverInput, ProverClient};

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::try_init().unwrap_or_default();
    let pre_elf_path =
        env::var("PRE_ELF_PATH").unwrap_or(env!("PRE_GUEST_TARGET_PATH").to_string());
    let elf_path = std::env::var("ELF_PATH").unwrap_or(env!("GUEST_TARGET_PATH").to_string());
    std::env::set_var("ELF_PATH", elf_path);

    // Here we can not run the groth16 setup, since we only have one segment.
    let (client_config, mut inner_prover_input) = ClientCfg::from_env(set_pre_guest_input);
    log::info!("create prover client");
    let prover_client = ProverClient::new(&client_config).await;
    let outer_seg_size = inner_prover_input.seg_size;
    let outer_elf = inner_prover_input.elf;
    inner_prover_input.seg_size = 0;
    inner_prover_input.elf = read(pre_elf_path).unwrap();
    inner_prover_input.composite_proof = true;

    let start = Instant::now();
    let proving_result = prover_client.prover.prove(&inner_prover_input, None).await;
    let mut receipts = vec![];
    let pre_elf_id: Vec<u8>;
    match proving_result {
        Ok(Some(prover_result)) => {
            prover_client
                .print_guest_execution_output(true, &prover_result)
                .expect("print pre guest program excution's output false.");
            receipts.push(prover_result.receipt);
            pre_elf_id = prover_result.elf_id;
            log::info!("pre_elf_id: {:?}", pre_elf_id);
        }
        Ok(None) => {
            bail!("Failed to generate proof due to void result.");
        }
        Err(e) => {
            log::info!("Failed to generate proof. error: {}", e);
            bail!("Failed to generate proof.");
        }
    }

    let end = Instant::now();
    let elapsed = end.duration_since(start);
    log::info!("Elapsed time: {:?} secs", elapsed.as_secs());

    let mut prover_input = ProverInput {
        elf: outer_elf,
        seg_size: outer_seg_size,
        execute_only: inner_prover_input.execute_only,
        receipts,
        snark_setup: inner_prover_input.snark_setup,
        proof_results_path: inner_prover_input.proof_results_path,
        ..Default::default()
    };

    set_guest_input(&mut prover_input, &pre_elf_id);

    let start = Instant::now();
    let proving_result = prover_client.prover.prove(&prover_input, None).await;
    match proving_result {
        Ok(Some(prover_result)) => {
            if !inner_prover_input.execute_only {
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

fn set_pre_guest_input(input: &mut ProverInput, _param: Option<&str>) {
    let num_bytes: usize = 1024;
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

fn set_guest_input(input: &mut ProverInput, elf_id: &Vec<u8>) {
    let num_bytes: usize = 1024;
    let pri_input = vec![5u8; num_bytes];
    let mut hasher = Sha256::new();
    hasher.update(&pri_input);
    let result = hasher.finalize();
    let output: [u8; 32] = result.into();

    // assume the  arg[0] = hash(public input), and the arg[1] = public input.
    let mut pre_pub_buf = Vec::new();
    bincode::serialize_into(&mut pre_pub_buf, &output).expect("public_input serialization failed");

    let mut hasher = Sha256::new();
    hasher.update(output);
    let result = hasher.finalize();
    let output: [u8; 32] = result.into();

    let public_input = output.to_vec();
    let mut pub_buf = Vec::new();
    bincode::serialize_into(&mut pub_buf, &public_input)
        .expect("public_input serialization failed");

    let mut elf_id_buf = Vec::new();
    bincode::serialize_into(&mut elf_id_buf, elf_id).expect("elf_id serialization failed");

    input.public_inputstream = pub_buf;
    input.private_inputstream = pre_pub_buf;
    input.receipt_inputs.push(elf_id_buf);
}
