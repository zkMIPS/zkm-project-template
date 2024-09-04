pub mod local;
pub mod network;
pub mod prover;

use network::prover::NetworkProver;
use prover::Prover;
use std::env;

pub struct ProverClient {
    pub prover: Box<dyn Prover>,
}

impl ProverClient {
    pub fn new() -> Self {
        #[allow(unreachable_code)]
        match env::var("ZKM_PROVER")
            .unwrap_or("network".to_string())
            .to_lowercase()
            .as_str()
        {
            // "local" => Self {
            //     prover: Box::new(CpuProver::new()),
            // },
            "network" => Self {
                prover: Box::new(NetworkProver::default()),
            },
            _ => panic!(
                "invalid value for ZKM_PROVER enviroment variable: expected 'local', or 'network'"
            ),
        }
    }

    // pub fn local() -> Self {
    //     Self { prover: Box::new(CpuProver::new()) }
    // }

    pub fn network() -> Self {
        Self {
            prover: Box::new(NetworkProver::default()),
        }
    }
}

impl Default for ProverClient {
    fn default() -> Self {
        Self::new()
    }
}
