pub mod input;
pub mod local;
pub mod network;
pub mod prover;

use std::fs;
use std::fs::File;
use std::io::Write;
use std::path::Path;

use anyhow::Context;
use anyhow::{bail, Result};
use common::file;
use local::prover::LocalProver;
use network::prover::NetworkProver;
use prover::{ClientCfg, Prover, ProverInput, ProverResult};
use serde::{Deserialize, Serialize};
use serde_json::to_writer;
use sha2::{Digest, Sha256};
use thiserror::Error;
use tokio::time::Duration;

pub use input::GuestInput;

pub struct ProverClient {
    pub prover: Box<dyn Prover>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PublicInputs {
    roots_before: Roots,
    roots_after: Roots,
    userdata: Vec<u8>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Roots {
    root: Vec<u64>,
}

pub const LOCAL_PROVER: &str = "local"; // for v0.3.0
pub const NETWORK_PROVER: &str = "network";

#[derive(Error, Debug)]
pub enum ZKMProverError {
    #[error("failed to generate proof: {0}")]
    ProvingError(anyhow::Error),
    #[error("failed to generate proof: the result is None")]
    ProvingResultNone,
    #[error("the result is empty, please set SEG_SIZE to half of its original value")]
    SegSizeTooBig(),
    #[error("io error: {0}")]
    IoError(std::io::Error),
}

pub fn is_local_prover(zkm_prover: &str) -> bool {
    zkm_prover.to_lowercase() == *LOCAL_PROVER
}

// Generic function to save serialized data to a JSON file
pub fn save_data_as_json<T: Serialize>(
    output_dir: &str,
    file_name: &str,
    data: &T,
) -> anyhow::Result<()> {
    // Create the output directory
    fs::create_dir_all(output_dir).unwrap();

    // Build the full file path
    let output_path = Path::new(&output_dir).join(file_name);

    // Open the file and write the data
    let mut file = File::create(&output_path).unwrap();
    to_writer(&mut file, data)?;

    log::info!("Data successfully written to file.");
    Ok(())
}

// Generic function to save data to a file
pub fn save_data_to_file<P: AsRef<Path>, D: AsRef<[u8]>>(
    output_dir: P,
    file_name: &str,
    data: D,
) -> anyhow::Result<()> {
    // Create the output directory
    let output_dir = output_dir.as_ref();
    log::info!("create dir: {}", output_dir.display());
    fs::create_dir_all(output_dir)?;

    // Build the full file path
    let output_path = output_dir.join(file_name);

    // Open the file and write the data
    let mut file = File::create(&output_path)?;
    file.write_all(data.as_ref())?;

    let bytes_written = data.as_ref().len();
    log::info!("Successfully written {} bytes.", bytes_written);

    Ok(())
}

pub fn update_public_inputs_with_bincode(
    public_inputstream: Vec<u8>,
    proof_public_inputs: &[u8],
) -> anyhow::Result<Option<PublicInputs>> {
    let mut hasher = Sha256::new();
    hasher.update(&public_inputstream);
    let result_hs = hasher.finalize();
    let output_hs: [u8; 32] = result_hs.into();

    let slice_bt: &[u8] = proof_public_inputs;
    let mut public_inputs: PublicInputs =
        serde_json::from_slice(slice_bt).expect("Failed to parse JSON");

    //1.check the userdata (from the proof) = hash(bincode(host's public_inputs)) ?
    let userdata = public_inputs.userdata;
    if userdata == output_hs {
        log::info!(" hash(bincode(pulic_input))1: {:?} ", &userdata);
        //2, update  userdata with bincode(host's  public_inputs).
        //the userdata is saved in the public_inputs.json.
        //the test contract  validates the public inputs in the snark proof file using this userdata.
        public_inputs.userdata = public_inputstream;
    } else if public_inputstream.is_empty() {
        log::info!(" hash(bincode(pulic_input))2: {:?} ", &userdata);
        //2', in this case, the bincode(public inputs) need setting to vec![0u8; 32].
        //the userdata is saved in the public_inputs.json.
        //the test contract  validates the public inputs in the snark proof file using this userdata.
        public_inputs.userdata = vec![0u8; 32];
    } else {
        log::info!(
            "public inputs's hash is different. the proof's is: {:?}, host's is :{:?} ",
            userdata,
            output_hs
        );
        bail!("Public inputs's hash does not match the proof's userdata.");
    }

    Ok(Some(public_inputs))
}

impl ProverClient {
    pub async fn new(client_config: &ClientCfg) -> Self {
        #[allow(unreachable_code)]
        match client_config.zkm_prover_type.as_str() {
            "local" => Self {
                prover: Box::new(LocalProver::new(client_config.key_path.as_ref().unwrap())),
            },
            "network" => {
                Self { prover: Box::new(NetworkProver::new(client_config).await.unwrap()) }
            }
            _ => panic!(
                "Invalid value for ZKM_PROVER enviroment variable: expected 'local', or 'network'"
            ),
        }
    }

    pub async fn form_env() -> Self {
        let client_config = ClientCfg::from_env();
        Self::new(&client_config).await
    }

    pub fn local(key_path: &str) -> Self {
        Self { prover: Box::new(LocalProver::new(key_path)) }
    }

    pub async fn network(client_config: &ClientCfg) -> Self {
        Self { prover: Box::new(NetworkProver::new(client_config).await.unwrap()) }
    }

    pub async fn prove(
        &self,
        input: &ProverInput,
        output_dir: Option<String>,
        timeout: Option<Duration>,
    ) -> Result<ProverResult, ZKMProverError> {
        let proving_result = self.prover.prove(input, timeout).await;
        match proving_result {
            Ok(Some(prover_result)) => {
                if !input.execute_only {
                    if prover_result.proof_with_public_inputs.is_empty() {
                        return Err(ZKMProverError::SegSizeTooBig());
                    }
                    if let Some(output_dir) = output_dir {
                        let output_path = Path::new(&output_dir);
                        let proof_result_path =
                            output_path.join("snark_proof_with_public_inputs.json");
                        let mut f = file::new(&proof_result_path.to_string_lossy());
                        f.write(prover_result.proof_with_public_inputs.as_slice())
                            .map_err(ZKMProverError::IoError)?;
                    }
                }
                Ok(prover_result)
            }
            Ok(_) => Err(ZKMProverError::ProvingResultNone),
            Err(e) => Err(ZKMProverError::ProvingError(e)),
        }
    }

    pub fn process_proof_results(
        &self,
        prover_result: &ProverResult,
        input: &ProverInput,
        zkm_prover_type: &str,
    ) -> anyhow::Result<()> {
        if prover_result.proof_with_public_inputs.is_empty() {
            if is_local_prover(zkm_prover_type) {
                //local proving
                log::info!("Fail: please try setting SEG_SIZE={}", input.seg_size / 2);
                bail!("SEG_SIZE is excessively large.");
            } else {
                //network proving
                log::info!(
                    "Fail: the SEG_SIZE={} out of the range of the proof network's.",
                    input.seg_size
                );

                bail!("SEG_SIZE is out of the range of the proof network's");
            }
        }
        //1.snark proof
        let output_dir = format!("{}/verifier", input.proof_results_path);
        save_data_to_file(
            &output_dir,
            "snark_proof_with_public_inputs.json",
            &prover_result.proof_with_public_inputs,
        )?;

        //2.handle the public inputs
        let public_inputs = update_public_inputs_with_bincode(
            input.public_inputstream.to_owned(),
            &prover_result.public_values,
        );
        match public_inputs {
            Ok(Some(inputs)) => {
                let output_dir = format!("{}/verifier", input.proof_results_path);
                log::info!("Save the public inputs:  ");
                save_data_as_json(&output_dir, "public_inputs.json", &inputs)?;
            }
            Ok(None) => {
                log::info!("Failed to update the public inputs.");
                bail!("Failed to update the public inputs.");
            }
            Err(e) => {
                log::info!("Failed to update the public inputs. error: {}", e);
                bail!("Failed to update the public inputs.");
            }
        }

        //3.contract,only for network proving
        if !is_local_prover(zkm_prover_type) {
            let output_dir = format!("{}/src", input.proof_results_path);
            log::info!("Save the verifier contract:  ");
            save_data_to_file(&output_dir, "verifier.sol", &prover_result.solidity_verifier)?;
        }

        log::info!("Generating proof successfully .The snark proof and contract are in the the path {}/{{verifier,src}} .", input.proof_results_path);

        Ok(())
    }

    // Generic function that automatically determines and prints based on the type T
    pub fn print_guest_execution_output(
        &self,
        has_output: bool,
        prover_result: &ProverResult,
    ) -> anyhow::Result<()> {
        if has_output {
            if prover_result.output_stream.is_empty() {
                log::info!(
                    "output_stream.len() is too short: {}",
                    prover_result.output_stream.len()
                );
                //return Err(anyhow::anyhow!("output_stream.len() is too short."));
                bail!("output_stream.len() is too short.");
            }
            log::info!("Executing the guest program  successfully.");
            log::info!("Guest's output messages: {:?}", prover_result.output_stream);
        } else {
            log::info!("Executing the guest program successfully without output any messages.")
        }

        Ok(())
    }

    // For handling struct types, we need another function
    pub fn print_guest_execution_output_struct<T>(
        &self,
        prover_result: &ProverResult,
    ) -> anyhow::Result<()>
    where
        T: serde::de::DeserializeOwned + std::fmt::Debug, // Here we restrict T to be deserializable
    {
        if prover_result.output_stream.is_empty() {
            log::info!("output_stream.len() is too short: {}", prover_result.output_stream.len());
            bail!("output_stream.len() is too short.");
        }
        log::info!("Executing the guest program  successfully.");
        // Deserialize the struct
        let ret_data: T = bincode::deserialize_from(prover_result.output_stream.as_slice())
            .context("Deserialization failed")?;

        log::info!("Guest's output messages: {:?}", ret_data);
        Ok(())
    }
}
