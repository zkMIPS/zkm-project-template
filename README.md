# ZKM Project Template

This is a template for creating an end-to-end ZKM project which can generate the EVM-Compatible proof and the on chain verification contract.

There are two ways to prove the guest program:

- Use your local machine
- Use ZKM proof network

## Running diagram

![image](assets/temp-run-diagram.png)

## Template code structure

> [!NOTE]
> The SDK has a libary(libsnark) which supports local proving. If the libsnark is required, please specify the features = ["snark"] in your Cargo.toml. To disable libsnark, set the environment variable NO_USE_SNARK to true when compiling the SDK.

```
├── Cargo.toml
├── LICENSE
├── Makefile
├── README.md
├── assets
│   └── temp-run-diagram.png
├── clippy.toml
├── contracts                   //Use Foundry to manage the verifier contract
│   ├── README.md
│   ├── foundry.toml
│   //...
├── guest-program               //Include Go and Rust examples
│   ├── README.md
│   ├── mem-alloc-vec
│   ├── sha2-go
│   └── sha2-rust
│  
├── host-program                //Generate the proof and verifier contracts  to the  guest programs
│   ├── Cargo.toml
│   ├── run-local-proving.sh
│   ├── run-network-proving.sh
│   ├── src
│      └── bin
│          └── zkm-prove.rs

├── install_mips_rust_tool
├── rust-toolchain.toml
├── sdk                         //Support proof network and local proof
    ├── Cargo.toml
    ├── build.rs
    └── src
       ├── lib.rs
       ├── local                //Generate the proof locally using the libsnark library.
       ├── network              //Generate the proof using ZKM Proof Network.
       ├── proto
       │   └── stage.proto
       └── prover.rs            //interface
```

## Local Proving Requirements

- Hardware: X86_64 CPU, 32 cores, 13GB memory (minimum)
- OS: Linux
- Rust: 1.81.0-nightly
- Go : 1.22.1
- Set up a local node for some blockchain(eg, sepolia)

## Network Proving Requirements

- Hardware: X86_64 CPU, 8 cores, 8G memory
- OS: Linux
- Rust: 1.81.0-nightly
- Go : 1.22.1
- CA certificate: ca.pem, ca.key
- [Register](https://www.zkm.io/apply) your address to use
- RPC for a blockchain (eg, sepolia)

> [!NOTE]
> All actions are assumed to be from the base directory `zkm-project-template`

## Running the project

### 0. Download the repo

```sh
git clone https://github.com/zkMIPS/zkm-project-template.git
```

### 1. Build the guest program ELF

Please refer to [this](guest-program/README.md) guide

### 2. Build the host program

```sh
sdk/src/local/libsnark/compile.sh  # compile snark library
cargo build --release              # build host programs
```

If the program executes successfully, it will generate one binary files in `target/release`: `zkm-prove`

> [!NOTE]
> The host program executes local proving when the environmental variable `ZKM_PROVER` is set to "local" and performs network proving when `ZKM_PROVER` is set to "network"

### 3. Generate groth16 proof and verifier contract

```sh
cd host-program
```

> [!NOTE]
> You can run the guest program without generating a proof by setting the environmental variable `EXECUTE_ONLY` to "true".https://github.com/zkMIPS/zkm/issues/152


### Example : `sha2-go`

This program takes struct Data as public input

#### Local Proving

Make any edits to [`run-local-proving.sh`](host-program/run-local-proving.sh) and run the program:

```sh
./run-local-proving.sh sha2-go
```

If successful, it will output a similar log:

##### **`sha2-go-local-proving.log`**

```
[2024-10-07T08:11:59Z INFO  zkm_prove] new prover client.
[2024-10-07T08:11:59Z INFO  zkm_prove] new prover client,ok.
[2024-10-07T08:11:59Z INFO  zkm_sdk::local::prover] calling request_proof.
[2024-10-07T08:11:59Z INFO  zkm_sdk::local::prover] calling wait_proof, proof_id=f8a8243f-8631-4f92-916c-754a438d8c57
[2024-10-07T08:11:59Z INFO  zkm_sdk::local::prover] waiting the proof result.
[2024-10-07T08:11:59Z INFO  zkm_emulator::utils] Split done 347823 : 446125
[2024-10-07T08:12:51Z INFO  zkm_sdk::local::util] Process segment /tmp/f8a8243f-8631-4f92-916c-754a438d8c57/input/segments/0
[2024-10-07T08:12:55Z INFO  zkm_prover::cpu::bootstrap_kernel] Bootstrapping took 21159 cycles
[2024-10-07T08:12:55Z INFO  zkm_prover::generation] CPU halted after 261370 cycles
[2024-10-07T08:12:55Z INFO  zkm_prover::generation] CPU trace padded to 262144 cycles
[2024-10-07T08:12:55Z INFO  zkm_prover::generation] Trace lengths (before padding): TraceCheckpoint { arithmetic_len: 131963, cpu_len: 262144, poseidon_len: 21158, poseidon_sponge_len: 21158, logic_len: 16739, memory_len: 1949524 }
[2024-10-07T08:13:42Z INFO  plonky2::util::timing] 50.7883s to prove root first
[2024-10-07T08:13:42Z INFO  zkm_sdk::local::util] Process segment /tmp/f8a8243f-8631-4f92-916c-754a438d8c57/input/segments/1
[2024-10-07T08:13:46Z INFO  zkm_prover::cpu::bootstrap_kernel] Bootstrapping took 21804 cycles
[2024-10-07T08:13:46Z INFO  zkm_prover::generation] CPU halted after 129420 cycles
[2024-10-07T08:13:46Z INFO  zkm_prover::generation] CPU trace padded to 131072 cycles
[2024-10-07T08:13:46Z INFO  zkm_prover::generation] Trace lengths (before padding): TraceCheckpoint { arithmetic_len: 44577, cpu_len: 131072, poseidon_len: 21803, poseidon_sponge_len: 21803, logic_len: 5164, memory_len: 1362331 }
[2024-10-07T08:14:20Z INFO  plonky2::util::timing] 37.3177s to prove root second
[2024-10-07T08:14:21Z INFO  plonky2::util::timing] 1.3277s to prove aggression
[2024-10-07T08:14:23Z INFO  zkm_sdk::local::util] proof size: 412755
[2024-10-07T08:14:29Z INFO  zkm_sdk::local::util] build finish
[2024-10-07T08:14:35Z INFO  plonky2x::backend::wrapper::wrap] Succesfully wrote common circuit data to common_circuit_data.json
[2024-10-07T08:14:35Z INFO  plonky2x::backend::wrapper::wrap] Succesfully wrote verifier data to verifier_only_circuit_data.json
[2024-10-07T08:14:35Z INFO  plonky2x::backend::wrapper::wrap] Succesfully wrote proof to proof_with_public_inputs.json
[2024-10-07T08:14:35Z INFO  plonky2::util::timing] 156.1002s to prove total time
08:14:39 INF compiling circuit
08:14:39 INF parsed circuit inputs nbPublic=1 nbSecret=11182
08:15:11 INF building constraint builder nbConstraints=5815132
Generating witness 2024-10-07 08:21:33.36586361 +0000 UTC m=+573.868286665
frontend.NewWitness cost time: 171 ms
Creating proof 2024-10-07 08:21:33.536971193 +0000 UTC m=+574.039394228
08:21:39 DBG constraint system solver done nbConstraints=5815132 took=5950.579212
08:21:47 DBG prover done acceleration=none backend=groth16 curve=bn254 nbConstraints=5815132 took=7963.861692
groth16.Prove cost time: 13914 ms
Verifying proof 2024-10-07 08:21:47.451908504 +0000 UTC m=+587.954331579
08:21:47 DBG verifier done backend=groth16 curve=bn254 took=1.707085
groth16.Verify cost time: 1 ms
before len of publicWitness:1
after len of publicWitness:2
08:21:47 DBG verifier done backend=groth16 curve=bn254 took=1.244218
[2024-10-07T08:21:47Z INFO  zkm_prove] Proof: successfully written 1260 bytes.
[2024-10-07T08:21:47Z INFO  zkm_prove] Contract: successfully written 10330 bytes.
[2024-10-07T08:21:47Z INFO  zkm_prove] Generating proof successfully .The proof file and verifier contract are in the the path contracts/verifier and contracts/src .
[2024-10-07T08:21:47Z INFO  zkm_prove] Elapsed time: 587 secs
```

The proof and contract file will be in the contracts/verifier and contracts/src

#### Network Proving

Make any edits to [`run-network-proving.sh`](host-program/run-network-proving.sh) and run the program:

```sh
./run-network-proving.sh sha2-rust
```

If successful, it will output a similar log:

##### **`sha2-go-network-proving.log`**

```

```

The proof and contract file will be in the contracts/verifier and contracts/src

#### 4. Deploy the Verifier Contract

If your system does not has Foundry, please install it:

```sh
curl -L https://foundry.paradigm.xyz | bash
```

Next, deploy the contract as detailed in [this](contracts/README.md) guide
