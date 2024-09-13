pub mod local;
pub mod network;
pub mod prover;

use local::prover::LocalProver;
use network::prover::NetworkProver;
use prover::Prover;
use std::env;

pub struct ProverClient {
    pub prover: Box<dyn Prover>,
}

impl ProverClient {
    pub async fn new() -> Self {
        #[allow(unreachable_code)]
        match env::var("ZKM_PROVER")
            .unwrap_or("network".to_string())
            .to_lowercase()
            .as_str()
        {
            "local" => Self {
                prover: Box::new(LocalProver::new()),
            },
            "network" => Self {
                prover: Box::new(NetworkProver::new().await.unwrap()),
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
