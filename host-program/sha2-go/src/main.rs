use std::env;
use serde::{Deserialize, Serialize};
use std::fs::read;

use std::time::Instant;
use zkm_sdk::{
    prover::ClientType, prover::ProverInput, ProverClient, NETWORK_PROVER,
};

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

    let seg_size = env::var("SEG_SIZE")
        .ok()
        .and_then(|seg| seg.parse::<u32>().ok())
        .unwrap_or(65536);

    let execute_only = env::var("EXECUTE_ONLY")
        .ok()
        .and_then(|seg| seg.parse::<bool>().ok())
        .unwrap_or(false);

    let elf_path = env::var("ELF_PATH").expect("ELF PATH is missed");
    let args_parameter = env::var("ARGS").unwrap_or("data-to-hash".to_string());
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
    set_guest_input(&mut prover_input, Some(&args_parameter));

    //the first executing the host will generate the pk and vk through setup().
    //if you want to generate the new vk , you should delete the files in the vk_path, then run the host program.
    prover_client
        .setup(&zkm_prover_type, &vk_path1, &prover_input)
        .await;

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
   // input.private_inputstream = pri_buf;
}