use zkm_sdk::{include_elf, HashableKey, ProverClient};

/// The ELF (executable and linkable format) file for the Succinct RISC-V zkVM.
pub const FIBONACCI_ELF: &[u8] = include_elf!("fibonacci");

fn main() {
    let prover = ProverClient::new();
    let (_, vk) = prover.setup(FIBONACCI_ELF);
    println!("program verification key: {}", vk.bytes32());
}
