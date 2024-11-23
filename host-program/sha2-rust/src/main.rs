use common::file;
//use hex;
use serde::{Deserialize, Serialize};
//use serde_json;
use serde_json::to_writer;
use sha2::{Digest, Sha256};
use std::env;
use std::fs;
use std::fs::read;
use std::fs::File;
use std::path::Path;
use std::time::Instant;
use zkm_sdk::{
    prover::ClientType, prover::InputProcessor, prover::ProverInput, prover::ProverResult,
    ProverClient,
};

pub const DEFAULT_PROVER_NETWORK_RPC: &str = "https://152.32.186.45:20002";
pub const DEFALUT_PROVER_NETWORK_DOMAIN: &str = "stage";
pub const LOCAL_PROVER: &str = "local";
pub const NETWORK_PROVER: &str = "network";


#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::try_init().unwrap_or_default();
    let args: Vec<String> = env::args().collect();
    let helper = || {
        log::info!(
            "Help: {} local or network",
            args[0]
        );
        std::process::exit(-1);
    };
    if args.len() < 2 {
        helper();
    }

    let zkm_prover = args[1];

    //let seg_size = env::var("SEG_SIZE").unwrap_or("8192".to_string());
    //let seg_size2 = seg_size.parse::<_>().unwrap_or(65536);
    let seg_size = env::var("SEG_SIZE")
        .ok()
        .and_then(|seg| seg.parse::<u32>().ok())
        .unwrap_or(65536);

    //let execute_only = env::var("EXECUTE_ONLY").unwrap_or("false".to_string());
    //let execute_only2 = execute_only.parse::<bool>().unwrap_or(false);
    let execute_only = env::var("EXECUTE_ONLY")
        .ok()
        .and_then(|seg| seg.parse::<bool>().ok())
        .unwrap_or(false);

    let elf_path = env::var("ELF_PATH").expect("ELF PATH is missed");
    let args_parameter = env::var("ARGS").unwrap_or("data-to-hash".to_string());
    let json_path = env::var("JSON_PATH").expect("JSON PATH is missing");
    let proof_results_path = env::var("PROOF_RESULTS_PATH").unwrap_or("../contracts".to_string());
    //let zkm_prover = env::var("ZKM_PROVER").expect("ZKM PROVER is missing");
    let vk_path1 = env::var("VERIFYING_KEY_PATH").unwrap_or("/tmp/input".to_string());

    //network proving
    let endpoint1 = env::var("ENDPOINT").unwrap_or(DEFAULT_PROVER_NETWORK_RPC.to_string());
    let ca_cert_path1 = env::var("CA_CERT_PATH").unwrap_or("".to_string());
    let cert_path1 = env::var("CERT_PATH").unwrap_or("".to_string());
    let key_path1 = env::var("KEY_PATH").unwrap_or("".to_string());
    let domain_name1 = env::var("DOMAIN_NAME").unwrap_or(DEFALUT_PROVER_NETWORK_DOMAIN.to_string());
    let private_key1 = env::var("PRIVATE_KEY").unwrap_or("".to_string());

    if zkm_prover.to_lowercase() == NETWORK_PROVER.to_string() && private_key1.is_empty() {
        //network proving
        log::info!("Please set the PRIVATE_KEY=");
        return Err("PRIVATE_KEY is not set".into());
    }

    let client_type: ClientType = ClientType {
        zkm_prover: zkm_prover.to_owned(),
        endpoint: endpoint1,
        ca_cert_path: ca_cert_path1,
        cert_path: cert_path1,
        key_path: key_path1,
        domain_name: domain_name1,
        private_key: private_key1,
        vk_path: vk_path1.to_owned(),
    };

    log::info!("new prover client.");
    let prover_client = ProverClient::new(&client_type).await;
    log::info!("new prover client,ok.");

    let mut prover_input = ProverInput {
        elf: read(elf_path).unwrap(),
        public_inputstream: vec![],
        private_inputstream: vec![],
        seg_size: seg_size,
        execute_only: execute_only,
        args: "".into(),
    };

    //the guest program has inputs
    set_guest_input(&mut prover_input, None);
    
    //the first executing the host will generate the pk and vk through setup().
    //if you want to generate the new vk , you should delete the files in the vk_path, then run the host program.
    setup(&zkm_prover, &vk_path1, &prover_client, &prover_input).await;

    let start = Instant::now();
    let proving_result = prover_client.prover.prove(&prover_input, None).await;
    match proving_result {
        Ok(Some(prover_result)) => {
            if !execute_only {
                //excute the guest program and generate the proof
                process_proof_results(
                    &prover_result,
                    &prover_input,
                    &proof_results_path,
                    &zkm_prover,
                )
                .expect("process proof results error");
            } else {
                //only excute the guest program without proof
                print_guest_excution_output(&args[1], &prover_result)
                    .expect("print guest program excution's output.");
            }
        }
        Ok(None) => {
            log::info!("Failed to generate proof.The result is None.");
            return Err("Failed to generate proof.".into());
        }
        Err(e) => {
            log::info!("Failed to generate proof. error: {}", e);
            return Err("Failed to generate proof.".into());
        }
    }

    let end = Instant::now();
    let elapsed = end.duration_since(start);
    log::info!("Elapsed time: {:?} secs", elapsed.as_secs());
    Ok(())
}

//If the vk or pk doesn't exist, it will run setup().
async fn setup(
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
            let _ = prover_client
                .prover
                .setup(vk_path, prover_input, None)
                .await;
            log::info!("setup successfully, the vk and pk all exist in the path:{}.", vk_path);
        }
    }
}


fn set_guest_input(input: &mut ProverInput, param: Option<&str>) {
        //input.public_inputstream.push(1);
        let num_bytes: usize = 1024; //Notice! : if this value is small, it will not generate the  proof.
        let pri_input = vec![5u8; num_bytes];
        let mut hasher = Sha256::new();
        hasher.update(&pri_input);
        let result = hasher.finalize();
        let output: [u8; 32] = result.into();

        // assume the  arg[0] = hash(public input), and the arg[1] = public input.
        let public_input = output.to_vec();
        let mut pub_buf = Vec::new();
        bincode::serialize_into(&mut pub_buf, &public_input)
            .expect("public_input serialization failed");

        let mut pri_buf = Vec::new();
        bincode::serialize_into(&mut pri_buf, &pri_input)
            .expect("private_input serialization failed");

        input.public_inputstream = pub_buf;
        input.private_inputstream = pri_buf;

}


fn process_proof_results(
    prover_result: &ProverResult,
    input: &ProverInput,
    proof_results_path: &String,
    zkm_prover: &str,
) -> anyhow::Result<()> {
    if prover_result.proof_with_public_inputs.is_empty() {
        if zkm_prover.to_lowercase() == LOCAL_PROVER.to_string() {
            //local proving
            log::info!("Fail: please try setting SEG_SIZE={}", input.seg_size / 2);
            return Err(anyhow::anyhow!("SEG_SIZE is excessively large."));
        } else {
            //network proving
            log::info!(
                "Fail: the SEG_SIZE={} out of the range of the proof network's.",
                input.seg_size
            );
            return Err(anyhow::anyhow!(
                "SEG_SIZE is out of the range of the proof network's."
            ));
        }
    }
    //1.snark proof
    let output_dir = format!("{}/verifier", proof_results_path);
    fs::create_dir_all(&output_dir)?;
    let output_path = Path::new(&output_dir);
    let proof_result_path = output_path.join("snark_proof_with_public_inputs.json");
    let mut f = file::new(&proof_result_path.to_string_lossy());
    match f.write(prover_result.proof_with_public_inputs.as_slice()) {
        Ok(bytes_written) => {
            log::info!("Proof: successfully written {} bytes.", bytes_written);
        }
        Err(e) => {
            log::info!("Proof: failed to write to file: {}", e);
            return Err(anyhow::anyhow!("Proof: failed to write to file."));
        }
    }

    //2.handle the public inputs
    let public_inputs = update_public_inputs_with_bincode(
        input.public_inputstream.clone(),
        &prover_result.public_values,
    );
    match public_inputs {
        Ok(Some(inputs)) => {
            let output_dir = format!("{}/verifier", proof_results_path);
            fs::create_dir_all(&output_dir)?;
            let output_path = Path::new(&output_dir);
            let public_inputs_path = output_path.join("public_inputs.json");
            let mut fp = File::create(public_inputs_path).expect("Unable to create file");
            //save the json file
            to_writer(&mut fp, &inputs).expect("Unable to write to public input file");
        }
        Ok(None) => {
            log::info!("Failed to update the public inputs.");
            return Err(anyhow::anyhow!("Failed to update the public inputs."));
        }
        Err(e) => {
            log::info!("Failed to update the public inputs. error: {}", e);
            return Err(anyhow::anyhow!("Failed to update the public inputs."));
        }
    }

    //3.contract
    let output_dir = format!("{}/src", proof_results_path);
    fs::create_dir_all(&output_dir)?;
    let output_path = Path::new(&output_dir);
    let contract_path = output_path.join("verifier.sol");
    let mut f = file::new(&contract_path.to_string_lossy());
    match f.write(prover_result.solidity_verifier.as_slice()) {
        Ok(bytes_written) => {
            log::info!("Contract: successfully written {} bytes.", bytes_written);
        }
        Err(e) => {
            log::info!("Contract: failed to write to file: {}", e);
            return Err(anyhow::anyhow!("Contract: failed to write to file."));
        }
    }
    log::info!("Generating proof successfully .The proof file and verifier contract are in the the path {}/{{verifier,src}} .", proof_results_path);

    Ok(())
}

fn print_guest_excution_output(
    guest_program: &str,
    prover_result: &ProverResult,
) -> anyhow::Result<()> {
    //The guest program outputs the basic type
    if prover_result.output_stream.is_empty() {
        log::info!(
            "output_stream.len() is too short: {}",
                    prover_result.output_stream.len()
            );
        return Err(anyhow::anyhow!("output_stream.len() is too short."));
    }
    log::info!("Executing the guest program  successfully.");
    log::info!("ret_data: {:?}", prover_result.output_stream);

    Ok(())
}

#[derive(Serialize, Deserialize, Debug)]
struct PublicInputs {
    roots_before: Roots,
    roots_after: Roots,
    userdata: Vec<u8>,
}

#[derive(Serialize, Deserialize, Debug)]
struct Roots {
    root: Vec<u64>,
}

fn update_public_inputs_with_bincode(
    public_inputstream: Vec<u8>,
    proof_public_inputs: &[u8],
) -> anyhow::Result<Option<PublicInputs>> {
    let mut hasher = Sha256::new();
    hasher.update(&public_inputstream);
    let result_hs = hasher.finalize();
    let output_hs: [u8; 32] = result_hs.into();

    let slice_bt: &[u8] = proof_public_inputs;
    let mut public_inputs: PublicInputs =
        serde_json::from_slice(slice_bt).expect("Failed to parse JSON");

    //1.check the userdata (from the proof) = hash(bincode(host's public_inputs)) ?
    let userdata = public_inputs.userdata;
    if userdata == output_hs {
        log::info!(" hash(bincode(pulic_input))1: {:?} ", &userdata);
        //2, update  userdata with bincode(host's  public_inputs).
        //the userdata is saved in the public_inputs.json.
        //the test contract  validates the public inputs in the snark proof file using this userdata.
        public_inputs.userdata = public_inputstream;
    } else if public_inputstream.is_empty() {
        log::info!(" hash(bincode(pulic_input))2: {:?} ", &userdata);
        //2', in this case, the bincode(public inputs) need setting to vec![0u8; 32].
        //the userdata is saved in the public_inputs.json.
        //the test contract  validates the public inputs in the snark proof file using this userdata.
        public_inputs.userdata = vec![0u8; 32];
    } else {
        log::info!(
            "public inputs's hash is different. the proof's is: {:?}, host's is :{:?} ",
            userdata,
            output_hs
        );
        return Err(anyhow::anyhow!(
            "Public inputs's hash does not match the proof's userdata."
        ));
    }

    Ok(Some(public_inputs))
}
