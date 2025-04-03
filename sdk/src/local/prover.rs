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
    key_path: String,
}

impl ProverTask {
    fn new(proof_id: &str, key_path: &str, input: &ProverInput) -> ProverTask {
        ProverTask {
            proof_id: proof_id.to_string(),
            input: input.clone(),
            result: None,
            is_done: false,
            key_path: key_path.to_string(),
        }
    }

    fn run(&mut self) {
        let mut result = ProverResult::default();
        let key_path = self.key_path.to_owned();
        let inputdir = format!("/tmp/{}/input", self.proof_id);
        let outputdir = format!("/tmp/{}/output", self.proof_id);
        log::debug!("key_path: {key_path}, input: {inputdir}, output: {outputdir}");
        fs::create_dir_all(&key_path).unwrap();
        fs::create_dir_all(&inputdir).unwrap();
        fs::create_dir_all(&outputdir).unwrap();
        let (should_agg, receipt, elf_id) =
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
        } else if !self.input.composite_proof {
            //excuting the setup_and_generate_sol_verifier
            if self.input.snark_setup {
                match zkm_recursion::groth16_setup(&inputdir) {
                    Ok(()) => {
                        log::info!("Succussfully setup_and_generate_sol_verifier at {inputdir}.")
                    }
                    Err(e) => {
                        log::info!("Error during setup_and_generate_sol_verifier: {}", e);
                        panic!("Failed to setup_and_generate_sol_verifier.");
                    }
                }
                let target_files = [
                    "proving.key",
                    "verifying.key",
                    "circuit",
                    "common_circuit_data.json",
                    "verifier_only_circuit_data.json",
                    "verifier.sol",
                ];
                std::fs::create_dir_all(format!("{}", key_path)).unwrap();
                target_files.iter().for_each(|f| {
                    std::fs::copy(format!("{inputdir}/{f}"), format!("{key_path}/{f}")).unwrap();
                });
            }

            match zkm_recursion::as_groth16(&key_path, &inputdir, &outputdir) {
                Ok(()) => {
                    result.stark_proof =
                        std::fs::read(format!("{}/proof_with_public_inputs.json", inputdir))
                            .unwrap();
                    result.proof_with_public_inputs =
                        std::fs::read(format!("{}/snark_proof_with_public_inputs.json", outputdir))
                            .unwrap();
                    //result.solidity_verifier =
                    //    std::fs::read(format!("{}/verifier.sol", outputdir)).unwrap();
                    result.public_values =
                        std::fs::read(format!("{}/public_values.json", inputdir)).unwrap();
                }
                Err(e) => {
                    log::error!("prove_snark error : {}", e);
                }
            }
        }
        if let Some(receipt) = receipt {
            result.receipt.clone_from(&receipt);
            result.elf_id.clone_from(&elf_id.unwrap());
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
    key_path: String,
}

impl LocalProver {
    pub fn new(key_path: &str) -> LocalProver {
        LocalProver { tasks: Arc::new(Mutex::new(HashMap::new())), key_path: key_path.to_string() }
    }
}

#[async_trait]
impl Prover for LocalProver {
    async fn request_proof<'a>(&self, input: &'a ProverInput) -> anyhow::Result<String> {
        let proof_id: String = uuid::Uuid::new_v4().to_string();
        let task: Arc<Mutex<ProverTask>> =
            Arc::new(Mutex::new(ProverTask::new(&proof_id, &self.key_path, input)));
        self.tasks.lock().unwrap().insert(proof_id.clone(), task.clone());
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
            log::info!("Waiting the proof result.");
            sleep(Duration::from_secs(30)).await;
        }
    }

    async fn prove<'a>(
        &self,
        input: &'a ProverInput,
        timeout: Option<Duration>,
    ) -> anyhow::Result<Option<ProverResult>> {
        let proof_id = self.request_proof(input).await?;
        log::info!("Calling wait_proof, proof_id={}", proof_id);
        self.wait_proof(&proof_id, timeout).await
    }
}
