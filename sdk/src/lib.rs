pub mod local;
pub mod network;
pub mod prover;

use local::prover::LocalProver;
use network::prover::NetworkProver;
use prover::Prover;
use std::env;

#[derive(Debug, Default, Clone)]
pub struct ClientType {
    pub zkm_prover: String,
    pub endpoint: String,
    pub ca_cert_path: String,
    pub cert_path: String,
    pub key_path: String,
    pub domain_name: String,
    pub private_key: String,
}

pub struct ProverClient {
    pub prover: Box<dyn Prover>,
}

impl ProverClient {
    pub async fn new(clientType: &ClientType) -> Self {
        #[allow(unreachable_code)]
        match clientType.zkm_prover
        {
            "local" => Self {
                prover: Box::new(LocalProver::new()),
            },
            "network" => Self {
                prover: Box::new(NetworkProver::new(clientType).await.unwrap()),
            },
            _ => panic!(
                "invalid value for ZKM_PROVER enviroment variable: expected 'local', or 'network'"
            ),
        }
    }

    pub fn local() -> Self {
        Self {
            prover: Box::new(LocalProver::new()),
        }
    }

    pub async fn network() -> Self {
        Self {
            prover: Box::new(NetworkProver::new().await.unwrap()),
        }
    }
}
