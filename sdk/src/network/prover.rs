use common::tls::Config;
use stage_service::stage_service_client::StageServiceClient;
use stage_service::{GenerateProofRequest, GetStatusRequest};
use std::env;
use std::time::Instant;
use tonic::transport::Endpoint;
use tonic::transport::{Channel, ClientTlsConfig};

use crate::prover::{Prover, ProverInput, ProverResult};
use ethers::signers::{LocalWallet, Signer};
use tokio::time::sleep;
use tokio::time::Duration;

use async_trait::async_trait;

pub mod stage_service {
    tonic::include_proto!("stage.v1");
}

use crate::network::prover::stage_service::Status;

pub const DEFAULT_PROVER_NETWORK_RPC: &str = "https://152.32.186.45:20002";
pub const DEFALUT_PROVER_NETWORK_DOMAIN: &str = "stage";

pub struct NetworkProver {
    pub stage_client: StageServiceClient<Channel>,
    pub wallet: LocalWallet,
}

impl NetworkProver {
    pub async fn new() -> anyhow::Result<NetworkProver> {
        let endpoint = env::var("ENDPOINT").unwrap_or(DEFAULT_PROVER_NETWORK_RPC.to_string());
        let ca_cert_path = env::var("CA_CERT_PATH").unwrap_or("".to_string());
        let cert_path = env::var("CERT_PATH").unwrap_or("".to_string());
        let key_path = env::var("KEY_PATH").unwrap_or("".to_string());
        let domain_name =
            env::var("DOMAIN_NAME").unwrap_or(DEFALUT_PROVER_NETWORK_DOMAIN.to_string());
        let private_key = env::var("PRIVATE_KEY").unwrap_or("".to_string());

        let ssl_config = if ca_cert_path.is_empty() {
            None
        } else {
            Some(Config::new(ca_cert_path, cert_path, key_path).await?)
        };

        let endpoint = match ssl_config {
            Some(config) => {
                let mut tls_config = ClientTlsConfig::new().domain_name(domain_name);
                if let Some(ca_cert) = config.ca_cert {
                    tls_config = tls_config.ca_certificate(ca_cert);
                }
                if let Some(identity) = config.identity {
                    tls_config = tls_config.identity(identity);
                }
                Endpoint::new(endpoint)?.tls_config(tls_config)?
            }
            None => Endpoint::new(endpoint)?,
        };
        let stage_client = StageServiceClient::connect(endpoint).await?;
        let wallet = private_key.parse::<LocalWallet>().unwrap();
        Ok(NetworkProver {
            stage_client,
            wallet,
        })
    }

    pub async fn sign_ecdsa(&self, request: &mut GenerateProofRequest) {
        let sign_data = match request.block_no {
            Some(block_no) => {
                format!(
                    "{}&{}&{}&{}",
                    request.proof_id, block_no, request.seg_size, request.args
                )
            }
            None => {
                format!("{}&{}&{}", request.proof_id, request.seg_size, request.args)
            }
        };
        let signature = self.wallet.sign_message(sign_data).await.unwrap();
        request.signature = signature.to_string();
    }

    pub async fn download_file(url: &str) -> anyhow::Result<Vec<u8>> {
        let response = reqwest::get(url).await?;
        let content = response.bytes().await?;
        Ok(content.to_vec())
    }
}

#[async_trait]
impl Prover for NetworkProver {
    async fn request_proof<'a>(&self, input: &'a ProverInput) -> anyhow::Result<String> {
        let proof_id = uuid::Uuid::new_v4().to_string();
        let mut request = GenerateProofRequest {
            proof_id: proof_id.clone(),
            elf_data: input.elf.clone(),
            seg_size: input.seg_size,
            public_input_stream: input.public_inputstream.clone(),
            private_input_stream: input.private_inputstream.clone(),
            execute_only: input.execute_only,
            ..Default::default()
        };
        self.sign_ecdsa(&mut request).await;
        let mut client = self.stage_client.clone();
        let response = client.generate_proof(request).await?.into_inner();
        Ok(response.proof_id)
    }

    async fn wait_proof<'a>(
        &self,
        proof_id: &'a str,
        timeout: Option<Duration>,
    ) -> anyhow::Result<Option<ProverResult>> {
        let start_time = Instant::now();
        let mut client = self.stage_client.clone();
        loop {
            if let Some(timeout) = timeout {
                if start_time.elapsed() > timeout {
                    return Err(anyhow::anyhow!("Proof generation timed out."));
                }
            }

            let get_status_request = GetStatusRequest {
                proof_id: proof_id.to_string(),
            };
            let get_status_response = client.get_status(get_status_request).await?.into_inner();

            match Status::from_i32(get_status_response.status as i32) {
                Some(Status::Computing) => {
                    //log::info!("generate_proof step: {}", get_status_response.step);
                    match get_status_response.step {
                        0 => log::info!("generate_proof : queuing the task."),
                        1 => log::info!("generate_proof : splitting the task."),
                        2 => log::info!("generate_proof : proving the task."),
                        3 => log::info!("generate_proof : aggregating the proof."),
                        4 => log::info!("generate_proof : aggregating the proof."),
                        5 => log::info!("generate_proof : finalizing the proof."),
                        6 => log::info!("generate_proof : completing the proof."),
                        i32::MIN..=-1_i32 | 7_i32..=i32::MAX => todo!(),
                    }
                    sleep(Duration::from_secs(30)).await;
                }
                Some(Status::Success) => {
                    let mut proof_result = ProverResult {
                        output_stream: get_status_response.output_stream,
                        proof_with_public_inputs: get_status_response.proof_with_public_inputs,
                        stark_proof: vec![],
                        solidity_verifier: vec![],
                        public_values: vec![],
                    };
                    if !get_status_response.stark_proof_url.is_empty() {
                        proof_result.stark_proof =
                            NetworkProver::download_file(&get_status_response.stark_proof_url)
                                .await?;
                    }
                    if !get_status_response.solidity_verifier_url.is_empty() {
                        proof_result.solidity_verifier = NetworkProver::download_file(
                            &get_status_response.solidity_verifier_url,
                        )
                        .await?;
                    }
                    if !get_status_response.public_values_url.is_empty() {
                        proof_result.public_values =
                            NetworkProver::download_file(&get_status_response.public_values_url)
                                .await?;
                    }
                    return Ok(Some(proof_result));
                }
                _ => {
                    log::error!(
                        "generate_proof failed status: {}",
                        get_status_response.status
                    );
                    return Ok(None);
                }
            }
        }
    }

    async fn prove<'a>(
        &self,
        input: &'a ProverInput,
        timeout: Option<Duration>,
    ) -> anyhow::Result<Option<ProverResult>> {
        log::info!("calling request_proof.");
        let proof_id = self.request_proof(input).await?;
        log::info!("calling wait_proof, proof_id={}", proof_id);
        self.wait_proof(&proof_id, timeout).await
    }
}
