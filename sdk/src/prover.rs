use std::default::Default;
use std::env;

use async_trait::async_trait;
use serde::Deserialize;
use serde::Serialize;
use tokio::time::Duration;

#[derive(Debug, Default, Clone)]
pub struct ClientCfg {
    pub zkm_prover_type: String,
    //pub setup_flag: bool,
    pub endpoint: Option<String>,
    pub ca_cert_path: Option<String>,
    pub cert_path: Option<String>,
    pub key_path: Option<String>,
    pub domain_name: Option<String>,
    pub proof_network_privkey: Option<String>,
}

impl ClientCfg {
    pub fn from_env() -> ClientCfg {
        let zkm_prover_type = env::var("ZKM_PROVER").expect("ZKM PROVER is missing");
        let endpoint = env::var("ENDPOINT").map_or(None, |endpoint| Some(endpoint.to_string()));
        let ca_cert_path = env::var("CA_CERT_PATH").map_or(None, |path| Some(path.to_string()));
        let cert_path = env::var("CERT_PATH").map_or(None, |x| Some(x.to_string()));
        let key_path = env::var("KEY_PATH").map_or(None, |x| Some(x.to_string()));
        let domain_name = Some(env::var("DOMAIN_NAME").unwrap_or("stage".to_string()));
        let proof_network_privkey =
            env::var("PROOF_NETWORK_PRVKEY").map_or(None, |x| Some(x.to_string()));

        ClientCfg {
            zkm_prover_type,
            endpoint,
            ca_cert_path,
            cert_path,
            key_path,
            domain_name,
            proof_network_privkey,
        }
    }
}

#[derive(Debug, Default, Deserialize, Serialize, Clone)]
pub struct ProverInput {
    pub elf: Vec<u8>,
    pub public_inputstream: Vec<u8>,
    pub private_inputstream: Vec<u8>,
    pub seg_size: u32,
    pub execute_only: bool,
    pub snark_setup: bool,
    pub composite_proof: bool,
    pub receipt_inputs: Vec<Vec<u8>>,
    pub receipts: Vec<Vec<u8>>,
    pub proof_results_path: String,
}

impl ProverInput {
    pub fn from_env() -> ProverInput {
        let seg_size =
            env::var("SEG_SIZE").ok().and_then(|seg| seg.parse::<u32>().ok()).unwrap_or(262144);
        let execute_only =
            env::var("EXECUTE_ONLY").ok().and_then(|seg| seg.parse::<bool>().ok()).unwrap_or(false);
        let snark_setup =
            env::var("SNARK_SETUP").ok().and_then(|seg| seg.parse::<bool>().ok()).unwrap_or(false);
        let proof_results_path =
            env::var("PROOF_RESULTS_PATH").unwrap_or("../contracts".to_string());

        ProverInput {
            seg_size,
            execute_only,
            snark_setup,
            proof_results_path,
            ..Default::default()
        }
    }

    pub fn set_elf(&mut self, elf: &[u8]) {
        self.elf = Vec::from(elf);
    }

    pub fn set_guest_input(&mut self, input: Vec<Vec<u8>>) {
        let mut pri_buf = Vec::new();
        bincode::serialize_into(&mut pri_buf, &input).expect("input serialization failed");
        self.private_inputstream = pri_buf;
    }
}

#[derive(Debug, Default, Deserialize, Serialize, Clone)]
pub struct ProverResult {
    pub total_steps: u64,
    pub split_cost: u64,
    pub output_stream: Vec<u8>,
    pub proof_with_public_inputs: Vec<u8>,
    pub stark_proof: Vec<u8>,
    pub solidity_verifier: Vec<u8>,
    pub public_values: Vec<u8>,
    pub receipt: Vec<u8>,
    pub elf_id: Vec<u8>,
}

#[async_trait]
pub trait Prover {
    async fn request_proof<'a>(&self, input: &'a ProverInput) -> anyhow::Result<String>;
    async fn wait_proof<'a>(
        &self,
        proof_id: &'a str,
        timeout: Option<Duration>,
    ) -> anyhow::Result<Option<ProverResult>>;

    async fn prove<'a>(
        &self,
        input: &'a ProverInput,
        timeout: Option<Duration>,
    ) -> anyhow::Result<Option<ProverResult>>;
}
