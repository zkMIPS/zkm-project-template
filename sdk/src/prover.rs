use async_trait::async_trait;
use serde::Deserialize;
use serde::Serialize;
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
    pub private_key: Option<String>,
}

#[derive(Debug, Default, Deserialize, Serialize, Clone)]
pub struct ProverInput {
    pub elf: Vec<u8>,
    pub public_inputstream: Vec<u8>,
    pub private_inputstream: Vec<u8>,
    pub seg_size: u32,
    pub execute_only: bool,
    pub args: String,
}

impl Default for ProverInput {
    pub fn default() -> Self {
        ProverInput {
            elf: Vec::new(),
            public_inputstream: Vec::new(),
            private_inputstream: Vec::new(),
            seg_size: 0,
            execute_only: false,
            args: "".to_owned(), // empty string
        }
    }
}

#[derive(Debug, Default, Deserialize, Serialize, Clone)]
pub struct ProverResult {
    pub output_stream: Vec<u8>,
    pub proof_with_public_inputs: Vec<u8>,
    pub stark_proof: Vec<u8>,
    pub solidity_verifier: Vec<u8>,
    pub public_values: Vec<u8>,
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
