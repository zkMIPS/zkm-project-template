use anyhow::bail;
use anyhow::Result;
use sha2::{Digest, Sha256};
use std::env;
use std::fs::read;
use std::time::Instant;
use zkm_sdk::{prover::ClientCfg, prover::ProverInput, ProverClient};

pub const DEFAULT_PROVER_NETWORK_RPC: &str = "https://152.32.186.45:20002";
pub const DEFALUT_PROVER_NETWORK_DOMAIN: &str = "stage";

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::try_init().unwrap_or_default();
    let seg_size = env::var("SEG_SIZE")
        .ok()
        .and_then(|seg| seg.parse::<u32>().ok())
        .unwrap_or(65536);

    let execute_only = env::var("EXECUTE_ONLY")
        .ok()
        .and_then(|seg| seg.parse::<bool>().ok())
        .unwrap_or(false);

    let elf_path = env::var("ELF_PATH").unwrap_or(env!("GUEST_TARGET_PATH").to_string());
    let pre_elf_path =
        env::var("PRE_ELF_PATH").unwrap_or(env!("PRE_GUEST_TARGET_PATH").to_string());
    let proof_results_path = env::var("PROOF_RESULTS_PATH").unwrap_or("../contracts".to_string());
    let vk_path = env::var("VERIFYING_KEY_PATH").unwrap_or("/tmp/input".to_string());

    //network proving
    let endpoint = env::var("ENDPOINT").unwrap_or(DEFAULT_PROVER_NETWORK_RPC.to_string());
    let ca_cert_path = env::var("CA_CERT_PATH").unwrap_or("".to_string());
    let cert_path = env::var("CERT_PATH").unwrap_or("".to_string());
    let key_path = env::var("KEY_PATH").unwrap_or("".to_string());
    let domain_name = env::var("DOMAIN_NAME").unwrap_or(DEFALUT_PROVER_NETWORK_DOMAIN.to_string());
    let private_key = env::var("PRIVATE_KEY").unwrap_or("".to_string());
    let zkm_prover_type = env::var("ZKM_PROVER").expect("ZKM PROVER is missing");

    let client_config: ClientCfg = ClientCfg {
        zkm_prover: zkm_prover_type.to_owned(),
        endpoint: Some(endpoint),
        ca_cert_path: Some(ca_cert_path),
        cert_path: Some(cert_path),
        key_path: Some(key_path),
        domain_name: Some(domain_name),
        private_key: Some(private_key),
        vk_path: vk_path.to_owned(),
    };

    log::info!("new prover client:");
    let prover_client = ProverClient::new(&client_config).await;
    log::info!("new prover client,ok.");

    let mut prover_input = ProverInput {
        elf: read(pre_elf_path).unwrap(),
        public_inputstream: vec![],
        private_inputstream: vec![],
        seg_size: 0,
        execute_only: false,
        precompile: true,
        receipt_inputs: vec![],
        receipts: vec![],
    };

    set_pre_guest_input(&mut prover_input, None);

    let start = Instant::now();
    let proving_result = prover_client.prover.prove(&prover_input, None).await;
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

    let mut prover_input = ProverInput {
        elf: read(elf_path).unwrap(),
        public_inputstream: vec![],
        private_inputstream: vec![],
        seg_size,
        execute_only,
        precompile: false,
        receipt_inputs: vec![],
        receipts,
    };

    set_guest_input(&mut prover_input, &pre_elf_id);

    let start = Instant::now();
    let proving_result = prover_client.prover.prove(&prover_input, None).await;
    match proving_result {
        Ok(Some(prover_result)) => {
            if !execute_only {
                //excute the guest program and generate the proof
                prover_client
                    .process_proof_results(
                        &prover_result,
                        &prover_input,
                        &proof_results_path,
                        &zkm_prover_type,
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
