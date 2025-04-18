use std::env;
use std::time::Instant;

use zkm_sdk_network::prover::ProverInput;
use zkm_sdk_network::{GuestInput, ProverClient};

/// The ELF (executable and linkable format) file for the zkMIPS zkVM.
pub const ELF: &[u8] = include_bytes!(env!("ZKM_ELF_fibonacci"));

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::try_init().unwrap_or_default();

    let output_dir = env::var("OUTPUT_DIR").unwrap_or(String::from("./output"));

    let client = ProverClient::form_env().await;

    let mut prover_input = ProverInput::from_env();
    prover_input.set_elf(ELF);

    // If the guest program doesn't have inputs, it doesn't need the setting.
    let mut guest_input = GuestInput::new();
    let n = 20u32;
    guest_input.write(&n);
    prover_input.set_guest_input(guest_input.buffer);

    let start = Instant::now();

    client.prove(&prover_input, &output_dir, None).await.expect("failed to generate proof");

    let end = Instant::now();
    let elapsed = end.duration_since(start);
    log::info!("Elapsed time: {:?} secs", elapsed.as_secs(),);
    Ok(())
}
