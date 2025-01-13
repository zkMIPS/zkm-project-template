use anyhow::bail;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::env;
use std::fs::read;
use std::time::Instant;
use zkm_sdk::{is_local_prover, prover::ClientCfg, prover::ProverInput, ProverClient};

const GUEST_TARGET_PATH: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../guest-program/sha2-go/sha2-go"
);

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

    let elf_path = env::var("ELF_PATH").unwrap_or(GUEST_TARGET_PATH.to_string());
    let args_parameter = env::var("ARGS").unwrap_or("data-to-hash".to_string());
    let proof_results_path = env::var("PROOF_RESULTS_PATH").unwrap_or("../contracts".to_string());
    let vk_path = env::var("VERIFYING_KEY_PATH").unwrap_or("/tmp/input".to_string());

    // network proving
    let endpoint = env::var("ENDPOINT").unwrap_or("".to_string());
    let ca_cert_path = env::var("CA_CERT_PATH").unwrap_or("".to_string());
    let cert_path = env::var("CERT_PATH").unwrap_or("".to_string());
    let key_path = env::var("KEY_PATH").unwrap_or("".to_string());
    let domain_name = env::var("DOMAIN_NAME").unwrap_or("".to_string());
    let private_key = env::var("PROOF_NETWORK_PRVKEY").unwrap_or("".to_string());
    let zkm_prover_type = env::var("ZKM_PROVER").expect("ZKM PROVER is missing");

    let mut client_config: ClientCfg =
        ClientCfg::new(zkm_prover_type.to_owned(), vk_path.to_owned());

    if !is_local_prover(&zkm_prover_type) {
        client_config.set_network(
            endpoint,
            ca_cert_path,
            cert_path,
            key_path,
            domain_name,
            private_key,
        );
    }

    let prover_client = ProverClient::new(&client_config).await;
    log::info!("new prover client,ok.");

    let mut prover_input = ProverInput {
        elf: read(elf_path).unwrap(),
        seg_size,
        execute_only,
        ..Default::default()
    };

    // If the guest program does't have inputs, it does't need the setting.
    set_guest_input(&mut prover_input, Some(&args_parameter));

    // excuting the setup_and_generate_sol_verifier
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
                //the sha2-go guest program has outputs messages, which are struct type.
                prover_client
                    .print_guest_execution_output_struct::<Data>(&prover_result)
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

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum DataId {
    TYPE1,
    TYPE2,
    TYPE3,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Data {
    pub input1: [u8; 10],
    pub input2: u8,
    pub input3: i8,
    pub input4: u16,
    pub input5: i16,
    pub input6: u32,
    pub input7: i32,
    pub input8: u64,
    pub input9: i64,
    pub input10: Vec<u8>,
    pub input11: DataId,
    pub input12: String,
}

impl Default for Data {
    fn default() -> Self {
        Self::new()
    }
}

impl Data {
    pub fn new() -> Self {
        let array = [1u8, 2u8, 3u8, 4u8, 5u8, 6u8, 7u8, 8u8, 9u8, 10u8];
        Self {
            input1: array,
            input2: 0x11u8,
            input3: -1i8,
            input4: 0x1122u16,
            input5: -1i16,
            input6: 0x112233u32,
            input7: -1i32,
            input8: 0x1122334455u64,
            input9: -1i64,
            input10: array[1..3].to_vec(),
            input11: DataId::TYPE3,
            input12: "hello".to_string(),
        }
    }
}

fn set_guest_input(input: &mut ProverInput, args: Option<&str>) {
    // assume the  arg[0] is the hash(input)(which is a public input), and the arg[1] is the input.
    let args: Vec<&str> = args.expect("args false").split_whitespace().collect();
    assert_eq!(args.len(), 2);
    let mut data = Data::new();
    // Fill in the input data
    data.input10 = hex::decode(args[0]).unwrap();
    data.input12 = args[1].to_string();
    let mut pub_buf = Vec::new();
    bincode::serialize_into(&mut pub_buf, &data).expect("serialization failed");

    input.public_inputstream = pub_buf;
}
