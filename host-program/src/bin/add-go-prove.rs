use common::file;
use serde::{Deserialize, Serialize};
use std::env;
use std::fs::read;
use std::path::Path;
use std::time::Instant;
use zkm_sdk::{prover::ProverInput, ProverClient};

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

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::try_init().unwrap_or_default();
    log::info!("new prover client.");
    let prover_client = ProverClient::new().await; //ENV: ZKM_PROVER=local
    log::info!("new prover client,ok.");
    let seg_size = env::var("SEG_SIZE").unwrap_or("131072".to_string());
    let seg_size2 = seg_size.parse::<_>().unwrap_or(131072);
    let execute_only = env::var("EXECUTE_ONLY").unwrap_or("false".to_string());
    let execute_only2 = execute_only.parse::<bool>().unwrap_or(false);
    let elf_path =
        env::var("ELF_PATH").unwrap_or("guest-program/mips-elf/zkm-mips-elf-add-go".to_string());

    let private_input_path = env::var("PRIVATE_INPUT_PATH").unwrap_or("".to_string());
    let output_dir = env::var("OUTPUT_DIR").unwrap_or("/tmp/zkm".to_string());
    let data = Data::new();
    let mut buf = Vec::new();
    bincode::serialize_into(&mut buf, &data).expect("serialization failed");
    let input = ProverInput {
        elf: read(elf_path).unwrap(),
        //public_inputstream: read(public_input_path).unwrap_or("".into()),
        public_inputstream: buf,
        private_inputstream: read(private_input_path).unwrap_or("".into()),
        seg_size: seg_size2,
        execute_only: execute_only2,
    };
    let start = Instant::now();
    let proving_result = prover_client.prover.prove(&input, None).await;
    match proving_result {
        Ok(Some(prover_result)) => {
            if execute_only2 == false {
                log::info!("Generating proof successfully .The proof file and verifier contract are in the path {}.",&output_dir);
                let output_path = Path::new(&output_dir);
                let proof_result_path = output_path.join("snark_proof_with_public_inputs.json");
                let _ = file::new(&proof_result_path.to_string_lossy())
                    .write(prover_result.proof_with_public_inputs.as_slice());
                //contract
                let output_path = Path::new(&output_dir);
                let contract_path = output_path.join("verifier.sol");
                let _ = file::new(&contract_path.to_string_lossy())
                    .write(prover_result.solidity_verifier.as_slice());
            } else {
                if prover_result.output_stream.len() == 0 {
                    log::info!("output_stream.len() is too short: {}",prover_result.output_stream.len());
                    return Ok(());
                }
                log::info!("Executing the guest program  successfully, result:");
                let ret_data: Data = bincode::deserialize_from(prover_result.output_stream.as_slice())
                .expect("deserialization failed");
                log::info!("ret_data: {:?}", ret_data);
                //let mut input1 = [0u8; 10];
                //input1.copy_from_slice(prover_result.output_stream[..10]);
               /* let input1: [u8; 10] = prover_result.output_stream[..10].try_into().expect("Slice has incorrect length");
                log::info!("input1: {:?}", input1);
                let input2 = prover_result.output_stream[10];
                log::info!("input2: {}", input2);
                let input3 = prover_result.output_stream[11] as i8;
                log::info!("input3: {}", input3);
                let input4 = u16::from_be_bytes(prover_result.output_stream[12..14].try_into().map_err(|_| "Invalid data")?);
                log::info!("input4: {}", input4);
                let input5 = i16::from_be_bytes(prover_result.output_stream[14..16].try_into().map_err(|_| "Invalid data")?);
                log::info!("input5: {}", input5);
                let input6 = u32::from_be_bytes(prover_result.output_stream[16..20].try_into().map_err(|_| "Invalid data")?);
                log::info!("input6: {}", input6);
                let input7 = i32::from_be_bytes(prover_result.output_stream[20..24].try_into().map_err(|_| "Invalid data")?);
                log::info!("input7: {}", input7);
                let input8 = u64::from_be_bytes(prover_result.output_stream[24..32].try_into().map_err(|_| "Invalid data")?);
                log::info!("input8: {}", input8);
                let input9 = i64::from_be_bytes(prover_result.output_stream[32..40].try_into().map_err(|_| "Invalid data")?);
                log::info!("input9: {:?}", input9);
                let input10 = prover_result.output_stream[40..].to_vec();
                log::info!("input10: {:?}", input10);*/
            }
            
        }
        Ok(None) => {
            log::info!("Failed to generate proof.The result is None.");
        }
        Err(e) => {
            log::info!("Failed to generate proof. error: {}", e);
            return Ok(());
        }
    }

    let end = Instant::now();
    let elapsed = end.duration_since(start);
    log::info!("Elapsed time: {:?} secs", elapsed.as_secs());
    Ok(())
}
