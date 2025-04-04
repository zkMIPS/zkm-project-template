use async_trait::async_trait;
use serde::Deserialize;
use serde::Serialize;
use std::default::Default;
use std::env;
use std::fs::read;
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
    pub fn from_env(
        set_guest_input: fn(&mut ProverInput, Option<&str>),
    ) -> (ClientCfg, ProverInput) {
        let seg_size =
            env::var("SEG_SIZE").ok().and_then(|seg| seg.parse::<u32>().ok()).unwrap_or(65536);

        let execute_only =
            env::var("EXECUTE_ONLY").ok().and_then(|seg| seg.parse::<bool>().ok()).unwrap_or(false);

        let snark_setup =
            env::var("SNARK_SETUP").ok().and_then(|seg| seg.parse::<bool>().ok()).unwrap_or(false);

        let guest_input = env::var("ARGS").ok();
        let elf_path = env::var("ELF_PATH").expect("ELF not found");
        let proof_results_path =
            env::var("PROOF_RESULTS_PATH").unwrap_or("../contracts".to_string());
        let zkm_prover_type = env::var("ZKM_PROVER").expect("ZKM PROVER is missing");

        // network proving
        let endpoint = env::var("ENDPOINT").map_or(None, |endpoint| Some(endpoint.to_string()));
        let ca_cert_path = env::var("CA_CERT_PATH").map_or(None, |path| Some(path.to_string()));
        let cert_path = env::var("CERT_PATH").map_or(None, |x| Some(x.to_string()));
        let key_path = env::var("KEY_PATH").map_or(None, |x| Some(x.to_string()));
        let domain_name = Some(env::var("DOMAIN_NAME").unwrap_or("stage".to_string()));
        let proof_network_privkey =
            env::var("PROOF_NETWORK_PRVKEY").map_or(None, |x| Some(x.to_string()));

        let mut prover_input = ProverInput {
            elf: read(elf_path).expect("Read ELF error"),
            seg_size,
            execute_only,
            snark_setup,
            proof_results_path,
            ..Default::default()
        };

        //If the guest program does't have inputs, it does't need the setting.
        set_guest_input(&mut prover_input, guest_input.as_deref());

        (
            ClientCfg {
                zkm_prover_type,
                endpoint,
                ca_cert_path,
                cert_path,
                key_path,
                domain_name,
                proof_network_privkey,
            },
            prover_input,
        )
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
