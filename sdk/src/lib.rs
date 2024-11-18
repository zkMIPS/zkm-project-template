pub mod local;
pub mod network;
pub mod prover;

use local::prover::LocalProver;
use network::prover::NetworkProver;
use prover::{Prover, ClientType};
use std::env;


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

    pub async fn network(clientType: &ClientType) -> Self {
        Self {
            prover: Box::new(NetworkProver::new(clientType).await.unwrap()),
        }
    }
}
