use crate::prover::{Prover, ProverInput, ProverResult};
use async_trait::async_trait;
use std::collections::HashMap;
use std::fs;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use std::time::Instant;
use tokio::time::sleep;

pub struct ProverTask {
    proof_id: String,
    input: ProverInput,
    result: Option<ProverResult>,
    is_done: bool,
}

impl ProverTask {
    fn new(proof_id: &str, input: &ProverInput) -> ProverTask {
        ProverTask {
            proof_id: proof_id.to_string(),
            input: input.clone(),
            result: None,
            is_done: false,
        }
    }

    fn run(&mut self) {
        let mut result = ProverResult::default();
        let inputdir = format!("/tmp/{}/input", self.proof_id);
        let outputdir = format!("/tmp/{}/output", self.proof_id);
        fs::create_dir_all(&inputdir).unwrap();
        fs::create_dir_all(&outputdir).unwrap();
        let should_agg =
            crate::local::stark::prove_stark(&self.input, &inputdir, &mut result).unwrap();
        if self.input.execute_only {
            result.proof_with_public_inputs = vec![];
            result.stark_proof = vec![];
            result.solidity_verifier = vec![];
        } else if !should_agg {
            log::info!(
                "There is only one segment with segment size {}, will skip the aggregation!",
                self.input.seg_size
            );
        } else if crate::local::snark::prove_snark(&inputdir, &outputdir) {
            result.stark_proof =
                std::fs::read(format!("{}/proof_with_public_inputs.json", inputdir)).unwrap();
            result.proof_with_public_inputs =
                std::fs::read(format!("{}/snark_proof_with_public_inputs.json", outputdir))
                    .unwrap();
            result.solidity_verifier =
                std::fs::read(format!("{}/verifier.sol", outputdir)).unwrap();
            result.public_values =
                std::fs::read(format!("{}/public_values.json", inputdir)).unwrap();
        } else {
            log::error!("Failed to generate snark proof.");
        }
        self.result = Some(result);
        self.is_done = true;
    }

    fn is_done(&self) -> bool {
        self.is_done
    }
}

pub struct LocalProver {
    tasks: Arc<Mutex<HashMap<String, Arc<Mutex<ProverTask>>>>>,
}

impl Default for LocalProver {
    fn default() -> Self {
        Self::new()
    }
}

impl LocalProver {
    pub fn new() -> LocalProver {
        LocalProver {
            tasks: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

#[async_trait]
impl Prover for LocalProver {
    async fn request_proof<'a>(&self, input: &'a ProverInput) -> anyhow::Result<String> {
        let proof_id: String = uuid::Uuid::new_v4().to_string();
        let task: Arc<Mutex<ProverTask>> = Arc::new(Mutex::new(ProverTask::new(&proof_id, input)));
        self.tasks
            .lock()
            .unwrap()
            .insert(proof_id.clone(), task.clone());
        thread::spawn(move || {
            task.lock().unwrap().run();
        });
        Ok(proof_id)
    }

    async fn wait_proof<'a>(
        &self,
        proof_id: &'a str,
        timeout: Option<Duration>,
    ) -> anyhow::Result<Option<ProverResult>> {
        let task = self.tasks.lock().unwrap().get(proof_id).unwrap().clone();
        let start_time = Instant::now();
        loop {
            if let Some(timeout) = timeout {
                if start_time.elapsed() > timeout {
                    return Err(anyhow::anyhow!("Proof generation timed out."));
                }
            }
            if task.lock().unwrap().is_done() {
                self.tasks.lock().unwrap().remove(proof_id);
                return Ok(task.lock().unwrap().result.clone());
            }
            log::info!("waiting the proof result.");
            sleep(Duration::from_secs(30)).await;
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
