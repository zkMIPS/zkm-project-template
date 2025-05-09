//! An end-to-end example of using the zkMIPS SDK to generate a proof of a program that can be executed
//! or have a core proof generated.
//!
//! You can run this script using the following command:
//! ```shell
//! RUST_LOG=info cargo run --release -- --execute
//! ```
//! or
//! ```shell
//! RUST_LOG=info cargo run --release -- --core
//! ```
//! or
//! ```shell
//! RUST_LOG=info cargo run --release -- --compressed
//! ```

use alloy_sol_types::SolType;
use clap::Parser;
use fibonacci_lib::PublicValuesStruct;
use zkm_sdk::{ProverClient, ZKMStdin, include_elf};

/// The ELF (executable and linkable format) file for the zkMIPS zkVM.
pub const FIBONACCI_ELF: &[u8] = include_elf!("fibonacci");

/// The arguments for the command.
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(long)]
    execute: bool,

    #[arg(long)]
    core: bool,

    #[arg(long)]
    compressed: bool,

    #[arg(long, default_value = "20")]
    n: u32,
}

fn main() {
    // Setup the logger.
    zkm_sdk::utils::setup_logger();
    dotenv::dotenv().ok();

    // Parse the command line arguments.
    let args = Args::parse();

    if args.execute == args.core && args.compressed == args.execute {
        eprintln!("Error: You must specify either --execute, --core, or --compress");
        std::process::exit(1);
    }

    // Setup the prover client.
    let client = ProverClient::new();

    // Setup the inputs.
    let mut stdin = ZKMStdin::new();
    stdin.write(&args.n);

    println!("n: {}", args.n);

    if args.execute {
        // Execute the program
        let (output, report) = client.execute(FIBONACCI_ELF, stdin).run().unwrap();
        println!("Program executed successfully.");

        // Read the output.
        let decoded = PublicValuesStruct::abi_decode(output.as_slice()).unwrap();
        let PublicValuesStruct { n, a, b } = decoded;
        println!("n: {}", n);
        println!("a: {}", a);
        println!("b: {}", b);

        let (expected_a, expected_b) = fibonacci_lib::fibonacci(n);
        assert_eq!(a, expected_a);
        assert_eq!(b, expected_b);
        println!("Values are correct!");

        // Record the number of cycles executed.
        println!("Number of cycles: {}", report.total_instruction_count());
    } else {
        // Setup the program for proving.
        let (pk, vk) = client.setup(FIBONACCI_ELF);

        // Generate the Core proof
        let proof = if args.core {
            client.prove(&pk, stdin).run().expect("failed to generate Core proof")
        } else {
            client
                .prove(&pk, stdin)
                .compressed()
                .run()
                .expect("failed to generate Compressed Proof")
        };
        println!("Successfully generated proof!");

        // Verify the proof.
        client.verify(&proof, &vk).expect("failed to verify proof");
        println!("Successfully verified proof!");
    }
}
