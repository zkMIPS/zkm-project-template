use async_trait::async_trait;
use serde::Deserialize;
use serde::Serialize;
use std::default::Default;
use tokio::time::Duration;

#[derive(Debug, Default, Clone)]
pub struct ClientCfg {
    pub zkm_prover: String,
    pub vk_path: String,
    //pub setup_flag: bool,
    pub endpoint: Option<String>,
    pub ca_cert_path: Option<String>,
    pub cert_path: Option<String>,
    pub key_path: Option<String>,
    pub domain_name: Option<String>,
    pub proof_network_privkey: Option<String>,
}

impl ClientCfg {
    pub fn new(zkm_prover_type: String, vk_path: String) -> ClientCfg {
        ClientCfg {
            zkm_prover: zkm_prover_type,
            vk_path,
            ..Default::default()
        }
    }

    pub fn set_network(
        &mut self,
        endpoint: String,
        ca_cert_path: String,
        cert_path: String,
        key_path: String,
        domain_name: String,
        private_key: String,
    ) {
        self.endpoint = Some(endpoint);
        self.ca_cert_path = Some(ca_cert_path);
        self.cert_path = Some(cert_path);
        self.key_path = Some(key_path);
        self.domain_name = Some(domain_name);
        self.proof_network_privkey = Some(private_key);
    }
}

#[derive(Debug, Default, Deserialize, Serialize, Clone)]
pub struct ProverInput {
    pub elf: Vec<u8>,
    pub public_inputstream: Vec<u8>,
    pub private_inputstream: Vec<u8>,
    pub seg_size: u32,
    pub execute_only: bool,
    pub precompile: bool,
    pub receipt_inputs: Vec<Vec<u8>>,
    pub receipts: Vec<Vec<u8>>,
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
    async fn setup_and_generate_sol_verifier<'a>(
        &self,
        vk_path: &'a str,
        input: &'a ProverInput,
        timeout: Option<Duration>,
    ) -> anyhow::Result<()>;
    async fn prove<'a>(
        &self,
        input: &'a ProverInput,
        timeout: Option<Duration>,
    ) -> anyhow::Result<Option<ProverResult>>;
}
