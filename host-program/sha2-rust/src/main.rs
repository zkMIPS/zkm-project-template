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

    let setup_flag = env::var("SETUP_FLAG")
        .ok()
        .and_then(|seg| seg.parse::<bool>().ok())
        .unwrap_or(false);

    let elf_path = env::var("ELF_PATH").expect("ELF PATH is missed");
    //let args_parameter = env::var("ARGS").unwrap_or("data-to-hash".to_string());
    //let json_path = env::var("JSON_PATH").expect("JSON PATH is missing");
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

    let mut prover_input = ProverInput::default();
    prover_input.elf = read(elf_path).unwrap();
    prover_input.seg_size = seg_size;
    prover_input.execute_only = execute_only;

    //If the guest program does't have inputs, it does't need the setting.
    set_guest_input(&mut prover_input, None);

    //excuting the setup_and_generate_sol_verifier
    if setup_flag {
        match prover_client
            .setup_and_generate_sol_verifier(&zkm_prover_type, &vk_path, &prover_input)
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
