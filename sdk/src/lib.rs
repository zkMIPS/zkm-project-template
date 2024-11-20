pub mod local;
pub mod network;
pub mod prover;

use local::prover::LocalProver;
use network::prover::NetworkProver;
use prover::{ClientType, Prover};

pub struct ProverClient {
    pub prover: Box<dyn Prover>,
}

impl ProverClient {
    pub async fn new(client_type: &ClientType) -> Self {
        #[allow(unreachable_code)]
        match client_type.zkm_prover.as_str() {
            "local" => Self {
                prover: Box::new(LocalProver::new()),
            },
            "network" => Self {
                prover: Box::new(NetworkProver::new(client_type).await.unwrap()),
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

    pub async fn network(client_type: &ClientType) -> Self {
        Self {
            prover: Box::new(NetworkProver::new(client_type).await.unwrap()),
        }
    }
}
