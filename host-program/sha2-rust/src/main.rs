use sha2::{Digest, Sha256};
use std::env;

use std::fs::read;

use std::time::Instant;
use zkm_sdk::{prover::ClientType, prover::ProverInput, ProverClient};

pub const DEFAULT_PROVER_NETWORK_RPC: &str = "https://152.32.186.45:20002";
pub const DEFALUT_PROVER_NETWORK_DOMAIN: &str = "stage";

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::try_init().unwrap_or_default();
    let args: Vec<String> = env::args().collect();
    let helper = || {
        log::info!("Help: {} local or network", args[0]);
        std::process::exit(-1);
    };
    if args.len() < 2 {
        helper();
    }

    let zkm_prover_type = &args[1];

    let seg_size1 = env::var("SEG_SIZE")
        .ok()
        .and_then(|seg| seg.parse::<u32>().ok())
        .unwrap_or(65536);

    let execute_only1 = env::var("EXECUTE_ONLY")
        .ok()
        .and_then(|seg| seg.parse::<bool>().ok())
        .unwrap_or(false);

    let setup_flag1 = env::var("SETUP_FLAG")
        .ok()
        .and_then(|seg| seg.parse::<bool>().ok())
        .unwrap_or(false);

    let elf_path = env::var("ELF_PATH").expect("ELF PATH is missed");
    //let args_parameter = env::var("ARGS").unwrap_or("data-to-hash".to_string());
    //let json_path = env::var("JSON_PATH").expect("JSON PATH is missing");
    let proof_results_path = env::var("PROOF_RESULTS_PATH").unwrap_or("../contracts".to_string());
    let vk_path1 = env::var("VERIFYING_KEY_PATH").unwrap_or("/tmp/input".to_string());

    //network proving
    let endpoint1 = env::var("ENDPOINT").unwrap_or(DEFAULT_PROVER_NETWORK_RPC.to_string());
    let ca_cert_path1 = env::var("CA_CERT_PATH").unwrap_or("".to_string());
    let cert_path1 = env::var("CERT_PATH").unwrap_or("".to_string());
    let key_path1 = env::var("KEY_PATH").unwrap_or("".to_string());
    let domain_name1 = env::var("DOMAIN_NAME").unwrap_or(DEFALUT_PROVER_NETWORK_DOMAIN.to_string());
    let private_key1 = env::var("PRIVATE_KEY").unwrap_or("".to_string());

    let client_type: ClientType = ClientType {
        zkm_prover: zkm_prover_type.to_owned(),
        endpoint: Some(endpoint1),
        ca_cert_path: Some(ca_cert_path1),
        cert_path: Some(cert_path1),
        key_path: Some(key_path1),
        domain_name: Some(domain_name1),
        private_key: Some(private_key1),
        vk_path: vk_path1.to_owned(),
    };

    log::info!("new prover client:");
    let prover_client = ProverClient::new(&client_type).await;
    log::info!("new prover client,ok.");

    let mut prover_input = ProverInput {
        elf: read(elf_path).unwrap(),
        public_inputstream: vec![],
        private_inputstream: vec![],
        seg_size: seg_size1,
        execute_only: execute_only1,
        args: "".into(),
    };

    //If the guest program does't have inputs, it does't need the setting.
    set_guest_input(&mut prover_input, None);

    //excuting the setup 
    if setup_flag1 {
        prover_client
        .setup(zkm_prover_type, &vk_path1, &prover_input)
        .await;

        return Ok(());
    }


    let start = Instant::now();
    let proving_result = prover_client.prover.prove(&prover_input, None).await;
    match proving_result {
        Ok(Some(prover_result)) => {
            if !execute_only1 {
                //excute the guest program and generate the proof
                prover_client
                    .process_proof_results(
                        &prover_result,
                        &prover_input,
                        &proof_results_path,
                        zkm_prover_type,
                    )
                    .expect("process proof results error");
            } else {
                //only excute the guest program without generating the proof.
                //the sha2-rust guest program has outputs messages, which are basic type.
                prover_client
                    .print_guest_execution_output(true, &prover_result)
                    .expect("print guest program excution's output.");
            }
        }
        Ok(None) => {
            log::info!("Failed to generate proof.The result is None.");
            return Err("Failed to generate proof.".into());
        }
        Err(e) => {
            log::info!("Failed to generate proof. error: {}", e);
            return Err("Failed to generate proof.".into());
        }
    }

    let end = Instant::now();
    let elapsed = end.duration_since(start);
    log::info!("Elapsed time: {:?} secs", elapsed.as_secs());
    Ok(())
}

fn set_guest_input(input: &mut ProverInput, _param: Option<&str>) {
    //input.public_inputstream.push(1);
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
