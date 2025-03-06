use stage_service::stage_service_client::StageServiceClient;
use stage_service::{GenerateProofRequest, GetStatusRequest};

use std::path::Path;
use std::time::Instant;
use tonic::transport::Endpoint;
use tonic::transport::{Certificate, Identity};
use tonic::transport::{Channel, ClientTlsConfig};

use crate::prover::{ClientCfg, Prover, ProverInput, ProverResult};
use ethers::signers::{LocalWallet, Signer};
use tokio::time::sleep;
use tokio::time::Duration;

use anyhow::{bail, Result};
use async_trait::async_trait;

#[derive(Clone)]
pub struct Config {
    pub ca_cert: Option<Certificate>,
    pub identity: Option<Identity>,
}

pub mod stage_service {
    tonic::include_proto!("stage.v1");
}

use crate::network::prover::stage_service::{Status, Step};

pub struct NetworkProver {
    pub stage_client: StageServiceClient<Channel>,
    pub wallet: LocalWallet,
}

impl NetworkProver {
    pub async fn new(client_config: &ClientCfg) -> Result<NetworkProver> {
        let ca_cert_path = client_config.ca_cert_path.to_owned().expect("CA_CERT_PATH must be set");
        let cert_path = client_config.cert_path.to_owned().expect("CERT_PATH must be set");
        let key_path = client_config.key_path.to_owned().expect("KEY_PATH must be set");
        let ssl_config = if ca_cert_path.is_empty() {
            None
        } else {
            let (ca_cert, identity) =
                get_cert_and_identity(ca_cert_path, cert_path, key_path).await?;
            Some(Config { ca_cert, identity })
        };
        let endpoint_para = client_config.endpoint.to_owned().expect("ENDPOINT must be set");
        let endpoint = match ssl_config {
            Some(config) => {
                let mut tls_config = ClientTlsConfig::new().domain_name(
                    client_config.domain_name.to_owned().expect("DOMAIN_NAME must be set"),
                );
                if let Some(ca_cert) = config.ca_cert {
                    tls_config = tls_config.ca_certificate(ca_cert);
                }
                if let Some(identity) = config.identity {
                    tls_config = tls_config.identity(identity);
                }
                Endpoint::new(endpoint_para.to_owned())?.tls_config(tls_config)?
            }
            None => Endpoint::new(endpoint_para.to_owned())?,
        };
        let private_key =
            client_config.proof_network_privkey.to_owned().expect("PRIVATE_KEY must be set");
        if private_key.is_empty() {
            panic!("Please set the PRIVATE_KEY");
        }
        let stage_client = StageServiceClient::connect(endpoint).await?;
        let wallet = private_key.parse::<LocalWallet>().unwrap();
        Ok(NetworkProver { stage_client, wallet })
    }

    pub async fn sign_ecdsa(&self, request: &mut GenerateProofRequest) {
        let sign_data = match request.block_no {
            Some(block_no) => {
                format!("{}&{}&{}", request.proof_id, block_no, request.seg_size)
            }
            None => {
                format!("{}&{}", request.proof_id, request.seg_size)
            }
        };
        let signature = self.wallet.sign_message(sign_data).await.unwrap();
        request.signature = signature.to_string();
    }

    pub async fn download_file(url: &str) -> Result<Vec<u8>> {
        let response = reqwest::get(url).await?;
        let content = response.bytes().await?;
        Ok(content.to_vec())
    }
}

#[async_trait]
impl Prover for NetworkProver {
    async fn request_proof<'a>(&self, input: &'a ProverInput) -> Result<String> {
        let proof_id = uuid::Uuid::new_v4().to_string();
        let mut request = GenerateProofRequest {
            proof_id: proof_id.clone(),
            elf_data: input.elf.clone(),
            seg_size: input.seg_size,
            public_input_stream: input.public_inputstream.clone(),
            private_input_stream: input.private_inputstream.clone(),
            execute_only: input.execute_only,
            precompile: input.composite_proof,
            ..Default::default()
        };
        for receipt in input.receipts.iter() {
            request.receipt.push(receipt.clone());
        }
        for receipt_input in input.receipt_inputs.iter() {
            request.receipt_input.push(receipt_input.clone());
        }
        self.sign_ecdsa(&mut request).await;
        let mut client = self.stage_client.clone();
        let response = client.generate_proof(request).await?.into_inner();
        Ok(response.proof_id)
    }

    async fn wait_proof<'a>(
        &self,
        proof_id: &'a str,
        timeout: Option<Duration>,
    ) -> Result<Option<ProverResult>> {
        let start_time = Instant::now();
        let mut split_start_time = Instant::now();
        let mut split_end_time = Instant::now();
        let mut client = self.stage_client.clone();
        let mut last_step = 0;
        loop {
            if let Some(timeout) = timeout {
                if start_time.elapsed() > timeout {
                    bail!("Proof generation timed out.");
                }
            }

            let get_status_request = GetStatusRequest { proof_id: proof_id.to_string() };
            let get_status_response = client.get_status(get_status_request).await?.into_inner();

            match Status::from_i32(get_status_response.status as i32) {
                Some(Status::Computing) => {
                    //log::info!("generate_proof step: {}", get_status_response.step);
                    match Step::from_i32(get_status_response.step) {
                        Some(Step::Init) => log::info!("generate_proof : queuing the task."),
                        Some(Step::InSplit) => {
                            if last_step == 0 {
                                split_start_time = Instant::now();
                            }
                            log::info!("generate_proof : splitting the task.");
                        }
                        Some(Step::InProve) => {
                            if last_step == 1 {
                                split_end_time = Instant::now();
                            }
                            log::info!("generate_proof : proving the task.");
                        }
                        Some(Step::InAgg) => log::info!("generate_proof : aggregating the proof."),
                        Some(Step::InAggAll) => {
                            log::info!("generate_proof : aggregating the proof.")
                        }
                        Some(Step::InFinal) => log::info!("generate_proof : finalizing the proof."),
                        Some(Step::End) => log::info!("generate_proof : completing the proof."),
                        None => todo!(),
                    }
                    last_step = get_status_response.step;
                    sleep(Duration::from_secs(30)).await;
                }
                Some(Status::Success) => {
                    let mut proof_result = ProverResult {
                        output_stream: get_status_response.output_stream,
                        proof_with_public_inputs: get_status_response.proof_with_public_inputs,
                        stark_proof: vec![],
                        solidity_verifier: vec![],
                        public_values: vec![],
                        total_steps: get_status_response.total_steps,
                        split_cost: split_end_time.duration_since(split_start_time).as_millis()
                            as u64,
                        receipt: get_status_response.receipt,
                        elf_id: get_status_response.elf_id,
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
                    log::error!("generate_proof failed status: {}", get_status_response.status);
                    bail!("generate_proof failed status: {}", get_status_response.status);
                }
            }
        }
    }

    async fn setup_and_generate_sol_verifier<'a>(
        &self,
        _vk_path: &'a str,
        _input: &'a ProverInput,
        _timeout: Option<Duration>,
    ) -> Result<()> {
        panic!("The proof network does not support the method!");
    }

    async fn prove<'a>(
        &self,
        input: &'a ProverInput,
        timeout: Option<Duration>,
    ) -> Result<Option<ProverResult>> {
        log::info!("calling request_proof.");
        let proof_id = self.request_proof(input).await?;
        log::info!("calling wait_proof, proof_id={}", proof_id);
        self.wait_proof(&proof_id, timeout).await
    }
}

async fn get_cert_and_identity(
    ca_cert_path: String,
    cert_path: String,
    key_path: String,
) -> Result<(Option<Certificate>, Option<Identity>)> {
    let ca_cert_path = Path::new(&ca_cert_path);
    let cert_path = Path::new(&cert_path);
    let key_path = Path::new(&key_path);
    let mut ca: Option<Certificate> = None;
    let mut identity: Option<Identity> = None;
    if ca_cert_path.is_file() {
        let ca_cert = tokio::fs::read(ca_cert_path)
            .await
            .unwrap_or_else(|err| panic!("Failed to read {:?}, err: {:?}", ca_cert_path, err));
        ca = Some(Certificate::from_pem(ca_cert));
    }

    if cert_path.is_file() && key_path.is_file() {
        let cert = tokio::fs::read(cert_path)
            .await
            .unwrap_or_else(|err| panic!("Failed to read {:?}, err: {:?}", cert_path, err));
        let key = tokio::fs::read(key_path)
            .await
            .unwrap_or_else(|err| panic!("Failed to read {:?}, err: {:?}", key_path, err));
        identity = Some(Identity::from_pem(cert, key));
    }
    Ok((ca, identity))
}
