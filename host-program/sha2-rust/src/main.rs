use common::file;
//use hex;
use serde::{Deserialize, Serialize};
//use serde_json;
use serde_json::to_writer;
use sha2::{Digest, Sha256};
use std::env;
use std::fs;
use std::fs::read;
use std::fs::File;
use std::path::Path;
use std::time::Instant;
use zkm_sdk::{
    prover::ClientType, prover::ProverInput, prover::ProverResult,
    ProverClient, LOCAL_PROVER, NETWORK_PROVER,
};

pub const DEFAULT_PROVER_NETWORK_RPC: &str = "https://152.32.186.45:20002";
pub const DEFALUT_PROVER_NETWORK_DOMAIN: &str = "stage";



#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::try_init().unwrap_or_default();
    let args: Vec<String> = env::args().collect();
    let helper = || {
        log::info!(
            "Help: {} local or network",
            args[0]
        );
        std::process::exit(-1);
    };
    if args.len() < 2 {
        helper();
    }

    let zkm_prover_type = &args[1];

    let seg_size = env::var("SEG_SIZE")
        .ok()
        .and_then(|seg| seg.parse::<u32>().ok())
        .unwrap_or(65536);

    let execute_only = env::var("EXECUTE_ONLY")
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

    if zkm_prover_type.to_lowercase() == NETWORK_PROVER.to_string() && private_key1.is_empty() {
        //network proving
        log::info!("Please set the PRIVATE_KEY=");
        return Err("PRIVATE_KEY is not set".into());
    }

    let client_type: ClientType = ClientType {
        zkm_prover: zkm_prover_type.to_owned(),
        endpoint: endpoint1,
        ca_cert_path: ca_cert_path1,
        cert_path: cert_path1,
        key_path: key_path1,
        domain_name: domain_name1,
        private_key: private_key1,
        vk_path: vk_path1.to_owned(),
    };

    let prover_client = ProverClient::new(&client_type).await;
    log::info!("new prover client,ok.");

    let mut prover_input = ProverInput {
        elf: read(elf_path).unwrap(),
        public_inputstream: vec![],
        private_inputstream: vec![],
        seg_size: seg_size,
        execute_only: execute_only,
        args: "".into(),
    };

    //If the guest program does't have inputs, it does't need the setting.
    set_guest_input(&mut prover_input, None);
    
    //the first executing the host will generate the pk and vk through setup().
    //if you want to generate the new vk , you should delete the files in the vk_path, then run the host program.
    prover_client.setup(&zkm_prover_type, &vk_path1, &prover_input).await;

    let start = Instant::now();
    let proving_result = prover_client.prover.prove(&prover_input, None).await;
    match proving_result {
        Ok(Some(prover_result)) => {
            if !execute_only {
                //excute the guest program and generate the proof
                prover_client.process_proof_results(
                    &prover_result,
                    &prover_input,
                    &proof_results_path,
                    &zkm_prover_type,
                )
                .expect("process proof results error");
            } else {
                //only excute the guest program without proof
                print_guest_excution_output(&prover_result)
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

fn set_guest_input(input: &mut ProverInput, param: Option<&str>) {
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
        bincode::serialize_into(&mut pri_buf, &pri_input)
            .expect("private_input serialization failed");

        input.public_inputstream = pub_buf;
        input.private_inputstream = pri_buf;

}

fn print_guest_excution_output(
    prover_result: &ProverResult,
) -> anyhow::Result<()> {
    //The guest program outputs the basic type
    if prover_result.output_stream.is_empty() {
        log::info!(
            "output_stream.len() is too short: {}",
                    prover_result.output_stream.len()
            );
        return Err(anyhow::anyhow!("output_stream.len() is too short."));
    }
    log::info!("Executing the guest program  successfully.");
    log::info!("ret_data: {:?}", prover_result.output_stream);

    Ok(())
}
