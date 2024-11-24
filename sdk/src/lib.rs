pub mod local;
pub mod network;
pub mod prover;

use common::file;
use local::prover::LocalProver;
use network::prover::NetworkProver;
use prover::{ClientType, Prover, ProverInput, ProverResult};
use serde::{Deserialize, Serialize};
use std::fs;
use std::fs::File;
use std::path::Path;

use serde_json::to_writer;
use sha2::{Digest, Sha256};

use anyhow::Context;
use bincode;

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

// Trait to check if T implements the Copy trait
pub trait IsBasicType: Copy {}

// Implement IsBasicType for all types that implement Copy
impl<T: Copy> IsBasicType for T {}

pub const LOCAL_PROVER: &str = "local";
pub const NETWORK_PROVER: &str = "network";

impl ProverClient {
    pub async fn new(client_type: &ClientType) -> Self {
        #[allow(unreachable_code)]
        match client_type.zkm_prover.as_str() {
            "local" => Self {
                prover: Box::new(LocalProver::new(&client_type.vk_path)),
            },
            "network" => Self {
                prover: Box::new(NetworkProver::new(client_type).await.unwrap()),
            },
            _ => panic!(
                "invalid value for ZKM_PROVER enviroment variable: expected 'local', or 'network'"
            ),
        }
    }

    pub fn local(vk_path: &str) -> Self {
        Self {
            prover: Box::new(LocalProver::new(vk_path)),
        }
    }

    pub async fn network(client_type: &ClientType) -> Self {
        Self {
            prover: Box::new(NetworkProver::new(client_type).await.unwrap()),
        }
    }

    //If the vk or pk doesn't exist, it will run setup().
    pub async fn setup(&self, zkm_prover: &str, vk_path: &str, prover_input: &ProverInput) {
        if zkm_prover.to_lowercase() == LOCAL_PROVER.to_string() {
            let pk_file = format!("{}/proving.key", vk_path);
            let vk_file = format!("{}/verifying.key", vk_path);

            let pathp = Path::new(&pk_file);
            let pathv = Path::new(&vk_file);

            if pathp.exists() && pathv.exists() {
                log::info!(
                    "The vk and pk all exist in the path:{} and don't need to setup.",
                    vk_path
                );
            } else {
                //setup the vk and pk for the first running local proving.
                log::info!("excuting the setup.");
                let _ = self.prover.setup(vk_path, prover_input, None).await;
                log::info!(
                    "setup successfully, the vk and pk all exist in the path:{}.",
                    vk_path
                );
            }
        }
    }

    pub fn process_proof_results(
        &self,
        prover_result: &ProverResult,
        input: &ProverInput,
        proof_results_path: &String,
        zkm_prover_type: &str,
    ) -> anyhow::Result<()> {
        if prover_result.proof_with_public_inputs.is_empty() {
            if zkm_prover_type.to_lowercase() == LOCAL_PROVER.to_string() {
                //local proving
                log::info!("Fail: please try setting SEG_SIZE={}", input.seg_size / 2);
                return Err(anyhow::anyhow!("SEG_SIZE is excessively large."));
            } else {
                //network proving
                log::info!(
                    "Fail: the SEG_SIZE={} out of the range of the proof network's.",
                    input.seg_size
                );
                return Err(anyhow::anyhow!(
                    "SEG_SIZE is out of the range of the proof network's."
                ));
            }
        }
        //1.snark proof
        let output_dir = format!("{}/verifier", proof_results_path);
        log::info!("save the snark proof:  ");
        Self::save_data_to_file(output_dir, "snark_proof_with_public_inputs.json", &prover_result.proof_with_public_inputs)?;
        /*fs::create_dir_all(&output_dir)?;
        let output_path = Path::new(&output_dir);
        let proof_result_path = output_path.join("snark_proof_with_public_inputs.json");
        let mut f = file::new(&proof_result_path.to_string_lossy());
        match f.write(prover_result.proof_with_public_inputs.as_slice()) {
            Ok(bytes_written) => {
                log::info!("Proof: successfully written {} bytes.", bytes_written);
            }
            Err(e) => {
                log::info!("Proof: failed to write to file: {}", e);
                return Err(anyhow::anyhow!("Proof: failed to write to file."));
            }
        }*/

        //2.handle the public inputs
        let public_inputs = Self::update_public_inputs_with_bincode(
            input.public_inputstream.to_owned(),
            &prover_result.public_values,
        );
        match public_inputs {
            Ok(Some(inputs)) => {
                let output_dir = format!("{}/verifier", proof_results_path);
                log::info!("save the public inputs:  ");
                Self::save_data_as_json(output_dir, "public_inputs.json", &inputs)?;
                /*fs::create_dir_all(&output_dir)?;
                let output_path = Path::new(&output_dir);
                let public_inputs_path = output_path.join("public_inputs.json");
                let mut fp = File::create(public_inputs_path).expect("Unable to create file");
                //save the json file
                to_writer(&mut fp, &inputs).expect("Unable to write to public input file");*/
            }
            Ok(None) => {
                log::info!("Failed to update the public inputs.");
                return Err(anyhow::anyhow!("Failed to update the public inputs."));
            }
            Err(e) => {
                log::info!("Failed to update the public inputs. error: {}", e);
                return Err(anyhow::anyhow!("Failed to update the public inputs."));
            }
        }

        //3.contract
        let output_dir = format!("{}/src", proof_results_path);
        log::info!("save the verifier contract:  ");
        Self::save_data_to_file(output_dir, "verifier.sol", &prover_result.solidity_verifier)?;
        /*fs::create_dir_all(&output_dir)?;
        let output_path = Path::new(&output_dir);
        let contract_path = output_path.join("verifier.sol");
        let mut f = file::new(&contract_path.to_string_lossy());
        match f.write(prover_result.solidity_verifier.as_slice()) {
            Ok(bytes_written) => {
                log::info!("Contract: successfully written {} bytes.", bytes_written);
            }
            Err(e) => {
                log::info!("Contract: failed to write to file: {}", e);
                return Err(anyhow::anyhow!("Contract: failed to write to file."));
            }
        }*/
        log::info!("Generating proof successfully .The proof file and verifier contract are in the the path {}/{{verifier,src}} .", proof_results_path);

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
            return Err(anyhow::anyhow!(
                "Public inputs's hash does not match the proof's userdata."
            ));
        }

        Ok(Some(public_inputs))
    }

    // Generic function that automatically determines and prints based on the type T
    pub fn print_guest_execution_output<T>(
        &self,
        has_output: bool,
        prover_result: &ProverResult,
    ) -> anyhow::Result<()>
    where
        T: IsBasicType, // Here we restrict T to be a basic type
    {
        if has_output {
            if prover_result.output_stream.is_empty() {
                log::info!(
                    "output_stream.len() is too short: {}",
                    prover_result.output_stream.len()
                );
                return Err(anyhow::anyhow!("output_stream.len() is too short."));
            }
            log::info!("Executing the guest program  successfully.");
            log::info!("ret_data: {:?}", prover_result.output_stream);
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
            log::info!(
                "output_stream.len() is too short: {}",
                prover_result.output_stream.len()
            );
            return Err(anyhow::anyhow!("output_stream.len() is too short."));
        }
        log::info!("Executing the guest program  successfully.");
        // Deserialize the struct
        let ret_data: T = bincode::deserialize_from(prover_result.output_stream.as_slice())
            .context("deserialization failed")?;

        log::info!("ret_data: {:?}", ret_data);
        Ok(())
    }

    // Generic function to save data to a file
    pub fn save_data_to_file<P: AsRef<Path>, D: AsRef<[u8]>>(
        base_path: P,
        file_name: &str,
        data: D,
    ) -> Result<()> {
        // Create the output directory
        let output_dir = base_path.as_ref().join(file_name.split('.').next().unwrap_or(""));
        create_dir_all(&output_dir).context("Failed to create output directory")?;

        // Build the full file path
        let output_path = output_dir.join(file_name);

        // Open the file and write the data
        let mut file = File::create(&output_path).context("Unable to create file")?;
        file.write_all(data.as_ref())
            .context("Failed to write to file")?;

        // Log the number of bytes written
        let bytes_written = data.as_ref().len();
        info!("Successfully written {} bytes.", bytes_written);

        Ok(())
    }

    // Generic function to save serialized data to a JSON file
    pub fn save_data_as_json<T: Serialize>(
        output_dir: &str,
        file_name: &str,
        data: &T,
    ) -> Result<()> {
        // Create the output directory
        //let output_dir = format!("{}/verifier", base_path);
        create_dir_all(&output_dir).context("Failed to create output directory")?;

        // Build the full file path
        let output_path = Path::new(&output_dir).join(file_name);

        // Open the file and write the data
        let mut file = File::create(&output_path).context("Unable to create file")?;
        to_writer(&mut file, data).context("Failed to write to file")?;

        info!("Data successfully written to file.");
        Ok(())
    }

}
