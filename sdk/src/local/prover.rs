use crate::prover::{Prover, ProverInput, ProverResult};
use anyhow::Context;
use async_trait::async_trait;
use std::collections::HashMap;
use std::fs;
use std::path::Path;
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
    vk_path: String,
}

impl ProverTask {
    fn new(proof_id: &str, vk_path: &str, input: &ProverInput) -> ProverTask {
        ProverTask {
            proof_id: proof_id.to_string(),
            input: input.clone(),
            result: None,
            is_done: false,
            vk_path: vk_path.to_string(),
        }
    }

    fn run(&mut self) {
        let mut result = ProverResult::default();
        let vk_path = self.vk_path.to_owned();
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
        } else if crate::local::snark::prove_snark(&vk_path, &inputdir, &outputdir)
            .expect("true or false")
        {
            result.stark_proof =
                std::fs::read(format!("{}/proof_with_public_inputs.json", inputdir)).unwrap();
            result.proof_with_public_inputs =
                std::fs::read(format!("{}/snark_proof_with_public_inputs.json", outputdir))
                    .unwrap();
            //result.solidity_verifier =
            //    std::fs::read(format!("{}/verifier.sol", outputdir)).unwrap();
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
    vk_path: String,
}

impl LocalProver {
    pub fn new(vk_path: &str) -> LocalProver {
        LocalProver {
            tasks: Arc::new(Mutex::new(HashMap::new())),
            vk_path: vk_path.to_string(),
            // setup_flag: flag,
        }
    }
}

pub fn delete_dir_contents<P: AsRef<Path>>(path: P) -> anyhow::Result<()> {
    for entry in fs::read_dir(path).context("Failed to read directory")? {
        let entry = entry.context("Failed to read directory entry")?;
        let path = entry.path();

        if path.is_dir() {
            delete_dir_contents(&path).context("Failed to delete directory contents")?;
        } else {
            fs::remove_file(&path).context("Failed to delete file")?;
        }
    }
    Ok(())
}

#[async_trait]
impl Prover for LocalProver {
    async fn request_proof<'a>(&self, input: &'a ProverInput) -> anyhow::Result<String> {
        let proof_id: String = uuid::Uuid::new_v4().to_string();
        let task: Arc<Mutex<ProverTask>> =
            Arc::new(Mutex::new(ProverTask::new(&proof_id, &self.vk_path, input)));
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

    async fn setup_and_generate_sol_verifier<'a>(
        &self,
        vk_path: &'a str,
        input: &'a ProverInput,
        _timeout: Option<Duration>,
    ) -> anyhow::Result<()> {
        let mut result = ProverResult::default();
        let path = Path::new(vk_path);
        if !path.is_dir() {
            fs::create_dir_all(vk_path).unwrap();
        }
        
        //delete_dir_contents(vk_path).context("Failed to clear input directory")?;
        let tem_dir = "/tmp/setup";
        let path = Path::new(tem_dir);
        if !path.is_dir() {
            fs::create_dir_all(tem_dir).unwrap();
        } else {
            delete_dir_contents(tem_dir).context("Failed to clear input directory")?;
        }
        
        let should_agg = crate::local::stark::prove_stark(input, tem_dir, &mut result).unwrap();
        if !should_agg {
            log::info!("Setup: generating the stark proof false, please check the SEG_SIZE or other parameters.");
            return Err(anyhow::anyhow!(
                "Setup: generating the stark proof false, please check the SEG_SIZE or other parameters!"));
        }

        match crate::local::snark::setup_and_generate_sol_verifier(tem_dir) {
            Ok(true) => {
                //copy the result files to vk_path
                //1. pk
                let src_path = Path::new(tem_dir);
                let src_file = src_path.join("proving.key");
                let dst_path = Path::new(vk_path);
                let dst_file = dst_path.join("proving.key");
                fs::copy(src_file, dst_file)?;

                //2. vk
                let src_file = src_path.join("verifying.key");
                let dst_file = dst_path.join("verifying.key");
                fs::copy(src_file, dst_file)?;

                //3. contract
                let src_file = src_path.join("verifier.sol");
                let dst_file = dst_path.join("verifier.sol");
                fs::copy(src_file, dst_file)?;

                //4. circuit
                let src_file = src_path.join("circuit");
                let dst_file = dst_path.join("circuit");
                fs::copy(src_file, dst_file)?;
                log::info!("setup_and_generate_sol_verifier successfully, the verify key and verifier contract are in the {}", vk_path);
                Ok(())
            }
            Ok(false) => Err(anyhow::anyhow!(
                "snark: setup_and_generate_sol_verifier failed!"
            )),
            Err(_) => todo!(),
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
