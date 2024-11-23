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
                prover: Box::new(LocalProver::new(&client_type.vk_path)),
            },
            "network" => Self {
                prover: Box::new(NetworkProver::new(client_type).await.unwrap()),
            },
            _ => panic!(
                "invalid value for ZKM_PROVER enviroment variable: expected 'local', or 'network'"
            ),
        }
    }

    pub fn local(vk_path: &str) -> Self {
        Self {
            prover: Box::new(LocalProver::new(vk_path)),
        }
    }

    pub async fn network(client_type: &ClientType) -> Self {
        Self {
            prover: Box::new(NetworkProver::new(client_type).await.unwrap()),
        }
    }

    //If the vk or pk doesn't exist, it will run setup().
    pub async fn setup(
        &self,
        zkm_prover: &str,
        vk_path: &str,
        prover_client: &ProverClient,
        prover_input: &ProverInput,
    ) {
        if zkm_prover.to_lowercase() == LOCAL_PROVER.to_string() {
            let pk_file = format!("{}/proving.key", vk_path);
            let vk_file = format!("{}/verifying.key", vk_path);

            let pathp = Path::new(&pk_file);
            let pathv = Path::new(&vk_file);

            if pathp.exists() && pathv.exists() {
                log::info!("The vk and pk all exist in the path:{} and don't need to setup.", vk_path);
            } else {
                //setup the vk and pk for the first running local proving.
                log::info!("excuting the setup.");
                let _ = self
                    .prover
                    .setup(vk_path, prover_input, None)
                    .await;
                log::info!("setup successfully, the vk and pk all exist in the path:{}.", vk_path);
            }
        }
}
}

