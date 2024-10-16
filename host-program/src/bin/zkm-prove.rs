use common::file;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::env;
use std::path::Path;
use std::time::Instant;
use zkm_sdk::{prover::ProverInput, ProverClient};

use std::fs::read;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::try_init().unwrap_or_default();
    let args: Vec<String> = env::args().collect();
    let helper = || {
        log::info!(
            "Help: {} sha2-rust | sha2-go | mem-alloc-vec | revme",
            args[0]
        );
        std::process::exit(-1);
    };
    if args.len() < 2 {
        helper();
    }

    log::info!("new prover client.");
    let prover_client = ProverClient::new().await;
    log::info!("new prover client,ok.");

    let seg_size = env::var("SEG_SIZE").unwrap_or("8192".to_string());
    let seg_size2 = seg_size.parse::<_>().unwrap_or(65536);
    let execute_only = env::var("EXECUTE_ONLY").unwrap_or("false".to_string());
    let execute_only2 = execute_only.parse::<bool>().unwrap_or(false);

    let input: ProverInput = match args[1].as_str() {
        "sha2-rust" => {
            set_sha2_rust_input(seg_size2, execute_only2).expect("set sha2-rust input error")
        }
        "sha2-go" => set_sha2_go_input(seg_size2, execute_only2).expect("set sha2-go input error"),
        "mem-alloc-vec" => set_mem_alloc_vec_input(seg_size2, execute_only2)
            .expect("set mem-alloc-vec input error"),
        "revme" => set_revme_input(seg_size2, execute_only2).expect("set revme input error"),
        _ => {
            helper();
            ProverInput {
                elf: "".into(),
                public_inputstream: "".into(),
                private_inputstream: "".into(),
                seg_size: 0,
                execute_only: false,
            }
        }
    };

    let start = Instant::now();
    let proving_result = prover_client.prover.prove(&input, None).await;
    match proving_result {
        Ok(Some(prover_result)) => {
            if !execute_only2 {
                if prover_result.proof_with_public_inputs.is_empty() {
                    log::info!(
                        "Fail: snark_proof_with_public_inputs.len() is : {}.Please try setting SEG_SIZE={}",
                        prover_result.proof_with_public_inputs.len(), seg_size2/2
                    );
                    return Err("SEG_SIZE is excessively large".into());
                }
                let output_dir = "../contracts/verifier".to_string();
                let output_path = Path::new(&output_dir);
                let proof_result_path = output_path.join("snark_proof_with_public_inputs.json");
                let mut f = file::new(&proof_result_path.to_string_lossy());
                match f.write(prover_result.proof_with_public_inputs.as_slice()) {
                    Ok(bytes_written) => {
                        log::info!("Proof: successfully written {} bytes.", bytes_written);
                    }
                    Err(e) => {
                        log::info!("Proof: failed to write to file: {}", e);
                        return Err("Proof: failed to write to file".into());
                    }
                }
                //contract
                let output_dir = "../contracts/src".to_string();
                let output_path = Path::new(&output_dir);
                let contract_path = output_path.join("verifier.sol");
                let mut f = file::new(&contract_path.to_string_lossy());
                match f.write(prover_result.solidity_verifier.as_slice()) {
                    Ok(bytes_written) => {
                        log::info!("Contract: successfully written {} bytes.", bytes_written);
                    }
                    Err(e) => {
                        log::info!("Contract: failed to write to file: {}", e);
                        return Err("Contract: failed to write to file".into());
                    }
                }
                log::info!("Generating proof successfully .The proof file and verifier contract are in the the path contracts/verifier and contracts/src .");
            } else {
                match args[1].as_str() {
                    "sha2-rust" => {
                        //The guest program returns the basic type
                        if prover_result.output_stream.is_empty() {
                            log::info!(
                                "output_stream.len() is too short: {}",
                                prover_result.output_stream.len()
                            );
                            return Err("output_stream.len() is too short".into());
                        }
                        log::info!("Executing the guest program  successfully.");
                        log::info!("ret_data: {:?}", prover_result.output_stream);
                    }
                    "sha2-go" => {
                        //If the guest program returns the structure, the result need the bincode::deserialize !
                        if prover_result.output_stream.is_empty() {
                            log::info!(
                                "output_stream.len() is too short: {}",
                                prover_result.output_stream.len()
                            );
                            return Err("output_stream.len() is too short".into());
                        }
                        log::info!("Executing the guest program  successfully.");
                        let ret_data: Data =
                            bincode::deserialize_from(prover_result.output_stream.as_slice())
                                .expect("deserialization failed");
                        log::info!("ret_data: {:?}", ret_data);
                    }
                    "mem-alloc-vec" => log::info!("Executing the guest program successfully."), //The guest program returns nothing.
                    "revme" => log::info!("Executing the guest program successfully."), //The guest program returns nothing.
                    _ => log::info!("Do nothing."),
                }
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

fn set_sha2_rust_input(seg_size_u: u32, execute_only_b: bool) -> anyhow::Result<ProverInput> {
    let elf_path = env::var("ELF_PATH").expect("ELF PATH is missed");
    let num_bytes: usize = 1024; //Notice! : if this value is small, it will not generate the snark proof.
    let pri_input = vec![5u8; num_bytes];
    let mut hasher = Sha256::new();
    hasher.update(&pri_input);
    let result = hasher.finalize();
    let output: [u8; 32] = result.into();
    // assume the  arg[0] is the hash(input)(which is a public input), and the arg[1] is the input.
    let public_input = output.to_vec();
    let mut pub_buf = Vec::new();
    bincode::serialize_into(&mut pub_buf, &public_input)
        .expect("public_input serialization failed");
    let mut pri_buf = Vec::new();
    bincode::serialize_into(&mut pri_buf, &pri_input).expect("private_input serialization failed");
    let input = ProverInput {
        elf: read(elf_path).unwrap(),
        public_inputstream: pub_buf,
        private_inputstream: pri_buf,
        seg_size: seg_size_u,
        execute_only: execute_only_b,
    };

    Ok(input)
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

fn set_sha2_go_input(seg_size_u: u32, execute_only_b: bool) -> anyhow::Result<ProverInput> {
    let elf_path = env::var("ELF_PATH").expect("ELF PATH is missed");
    let args = env::var("ARGS").unwrap_or("data-to-hash".to_string());
    // assume the  arg[0] is the hash(input)(which is a public input), and the arg[1] is the input.
    let args: Vec<&str> = args.split_whitespace().collect();
    assert_eq!(args.len(), 2);
    let mut data = Data::new();
    // Fill in the input data
    data.input10 = hex::decode(args[0]).unwrap();
    data.input12 = args[1].to_string();
    let mut buf = Vec::new();
    bincode::serialize_into(&mut buf, &data).expect("serialization failed");
    let input = ProverInput {
        elf: read(elf_path).unwrap(),
        public_inputstream: buf,
        private_inputstream: "".into(),
        seg_size: seg_size_u,
        execute_only: execute_only_b,
    };

    Ok(input)
}

fn set_mem_alloc_vec_input(seg_size_u: u32, execute_only_b: bool) -> anyhow::Result<ProverInput> {
    let elf_path = env::var("ELF_PATH").expect("ELF PATH is missed");
    let input = ProverInput {
        elf: read(elf_path).unwrap(),
        public_inputstream: "".into(),
        private_inputstream: "".into(),
        seg_size: seg_size_u,
        execute_only: execute_only_b,
    };

    Ok(input)
}

fn set_revme_input(seg_size_u: u32, execute_only_b: bool) -> anyhow::Result<ProverInput> {
    let elf_path = env::var("ELF_PATH").expect("ELF PATH is missed");
    let json_path = env::var("JSON_PATH").expect("JSON PATH is missing");
    let input = ProverInput {
        elf: read(elf_path).unwrap(),
        public_inputstream: read(json_path).unwrap(),
        private_inputstream: "".into(),
        seg_size: seg_size_u,
        execute_only: execute_only_b,
    };

    Ok(input)
}
