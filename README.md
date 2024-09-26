# ZKM Project Template

This is a template for creating an end-to-end ZKM project which can generate the EVM-Compatible proof and the on chain verification contract.

There are two ways to prove the guest program:

- Use your local machine
- Use ZKM proof network

## Running diagram

![image](https://github.com/user-attachments/assets/a420c018-7292-4a67-bba6-048c1bd17c77)

## Template code structure

> [!NOTE]
> The SDK has a libary(libsnark) which supports local proving. If the libsnark is required, please specify the features = ["snark"] in your Cargo.toml. To disable libsnark, set the environment variable NO_USE_SNARK to true when compiling the SDK.

```
├── Cargo.toml
├── LICENSE
├── Makefile
├── README.md
├── clippy.toml
├── contracts      //Use Foundry to manage the verifier contract
│   ├── README.md
│   ├── ...
├── guest-program  //Include two examples: one in Go  and the other in Rust
│   ├── README.md
│   ├── add-go
│   ├── mips-elf
│   └── revme
├── host-program   //Generate the proof and verifier contracts  to the  guest programs
│   ├── src
│   │   └── bin
│   │       ├── add-go-prove.rs
│   │       └── revme-prove.rs
│   ├── test-vectors
│   │   └── 244.json
├── install_mips_rust_tool
├── rust-toolchain.toml
├── sdk          //Support proof network and local proof
    ├── Cargo.toml
    ├── build.rs
    └── src
       ├── lib.rs
       ├── local    //Generate the proof locally using the libsnark library.
       ├── network  //Generate the proof using ZKM Proof Network.
       ├── proto
       │   └── stage.proto
       └── prover.rs //interface
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

If the program executes successfully, it will generate two binary files in `target/release`: `add-go-prove`,`revme-prove`

> [!NOTE]
> The host program executes local proving when the environmental variable `ZKM_PROVER` is set to "local" and performs network proving when `ZKM_PROVER` is set to "network"

### 3. Generate groth16 proof and verifier contract

```sh
cd host-program
```

> [!NOTE]
> You can run the guest program without generating a proof by setting the environmental variable `EXECUTE_ONLY` to "true".https://github.com/zkMIPS/zkm/issues/152

> For an `EXECUTE_ONLY` example, please refer to [add-go-prove.rs](host-program/src/bin/add-go-prove.rs)

### Example 1: `add-go`

This program takes struct Data as public input

#### Local Proving

Make any edits to [`run-add-go-local-proving.sh`](host-program/run-add-go-local-proving.sh) and run the program:

```sh
./run-add-go-local-proving.sh
```

If successful, it will output a similar log:

##### **`run-add-go-local-proving.log`**

```
[2024-09-14T14:09:57Z INFO  add_go_prove] new prover client.
[2024-09-14T14:09:57Z INFO  add_go_prove] new prover client,ok.
[2024-09-14T14:09:57Z INFO  zkm_sdk::local::prover] calling request_proof.
[2024-09-14T14:09:57Z INFO  zkm_sdk::local::prover] calling wait_proof, proof_id=46871d50-e00d-4aa0-a346-2bb172ca9dcb
[2024-09-14T14:09:57Z INFO  zkm_sdk::local::prover] waiting the proof result.

[2024-09-14T14:09:57Z INFO  zkm_emulator::utils] Split done 530872
[2024-09-14T14:10:48Z INFO  zkm_sdk::local::util] Process segment /tmp/46871d50-e00d-4aa0-a346-2bb172ca9dcb/input/segments/0
[2024-09-14T14:10:52Z INFO  zkm_prover::cpu::bootstrap_kernel] Bootstrapping took 21159 cycles
//...
//...
[2024-09-14T14:20:58Z INFO  add_go_prove] Generating proof successfully .The proof file and verifier contract are in the path /tmp/zkm.
[2024-09-14T14:20:58Z INFO  add_go_prove] Elapsed time: 660 secs
```

The proof and contract file will be in the `OUTPUT_DIR`

#### Network Proving

Make any edits to [`run-add-go-network-proving.sh`](host-program/run-add-go-network-proving.sh) and run the program:

```sh
./run-add-go-network-proving.sh
```

If successful, it will output a similar log:

##### **`add-go-network-proving.log`**

```
[2024-09-14T14:40:02Z INFO  add_go_prove] new prover client.
[2024-09-14T14:40:03Z INFO  add_go_prove] new prover client,ok.
[2024-09-14T14:40:03Z INFO  zkm_sdk::network::prover] calling request_proof.
[2024-09-14T14:40:05Z INFO  zkm_sdk::network::prover] calling wait_proof, proof_id=a3ea5791-1d08-4bb1-b3d0-0779dc831e9e
[2024-09-14T14:40:05Z INFO  zkm_sdk::network::prover] generate_proof : queuing the task.
[2024-09-14T14:40:35Z INFO  zkm_sdk::network::prover] generate_proof : proving the task.
[2024-09-14T14:41:05Z INFO  zkm_sdk::network::prover] generate_proof : finalizing the proof.
[2024-09-14T14:41:38Z INFO  add_go_prove] Generating proof successfully .The proof file and verifier contract are in the path /tmp/zkm.
[2024-09-14T14:41:38Z INFO  add_go_prove] Elapsed time: 95 secs
```

The proof and contract file will be in the `OUTPUT_DIR`

### Example 2: `revme`

This program takes a block of data as public input

#### Generating the public input about a specific block

> [!NOTE]
> The local node is the [GOAT](https://goat.network) test chain in the following example.

```sh
cd ~
git clone https://github.com/zkMIPS/revme
cd revme
RPC_URL=http://localhost:8545 CHAIN_ID=1337 BLOCK_NO=244 RUST_LOG=debug SUITE_JSON_PATH=./test-vectors/244.json cargo run --example process
```

If successfully, it will generate `244.json` in the path test-vectors

```sh
cp test-vectors/244.json zkm-project-template/host-program/test-vectors/
```

#### Local Proving

Make any edits to [`run-revme-local-proving.sh`](host-program/run-revme-local-proving.sh) and run the program:

```sh
./run-revme-local-proving.sh
```

If successful, it will output a similar log:

##### **`revme-local-proving.log`**

```
[2024-09-15T02:37:53Z INFO  revme_prove] new prover client.
[2024-09-15T02:37:53Z INFO  revme_prove] new prover client,ok.
[2024-09-15T02:37:53Z INFO  zkm_sdk::local::prover] calling request_proof.
[2024-09-15T02:37:53Z INFO  zkm_sdk::local::prover] calling wait_proof, proof_id=eb4c9f1a-cdff-4638-8a7f-5f835991942f
[2024-09-15T02:37:53Z INFO  zkm_sdk::local::prover] waiting the proof result.
[2024-09-15T02:37:54Z INFO  zkm_emulator::utils] Split done 15045962
[2024-09-15T02:38:44Z INFO  zkm_sdk::local::util] Process segment /tmp/eb4c9f1a-cdff-4638-8a7f-5f835991942f/input/segments/0
[2024-09-15T02:38:47Z INFO  zkm_prover::cpu::bootstrap_kernel] Bootstrapping took 8130 cycles
[2024-09-15T02:38:48Z INFO  zkm_prover::generation] CPU halted after 274150 cycles
[2024-09-15T02:38:48Z INFO  zkm_prover::generation] CPU trace padded to 524288 cycles
//...
[2024-09-15T03:40:40Z INFO  zkm_sdk::local::util] Process segment /tmp/eb4c9f1a-cdff-4638-8a7f-5f835991942f/input/segments/57
[2024-09-15T03:40:44Z INFO  zkm_prover::cpu::bootstrap_kernel] Bootstrapping took 15612 cycles
[2024-09-15T03:40:44Z INFO  zkm_prover::generation] CPU halted after 119366 cycles
[2024-09-15T03:40:45Z INFO  zkm_prover::generation] CPU trace padded to 131072 cycles
//...
[2024-09-15T03:48:11Z INFO  revme_prove] Generating proof successfully .The proof file and verifier contract are in the path /tmp/zkm.
[2024-09-15T03:48:11Z INFO  revme_prove] Elapsed time: 4217 secs
```

The proof and contract file will be in the `OUTPUT_DIR`

#### Network Proving

> [!NOTE]
> The proving network may sometimes experience high traffic, causing proof tasks to be queued for hours.

> The proving task requires several stages: queuing, splitting, proving, aggregating and finalizing. Each stage involves a varying duration.

Make any edits to [`run-revme-remote-proving.sh`](host-program/run-revme-remote-proving.sh) and run the program:

```
./run-revme-network-proving.sh
```

If successful, it will output a similar log:

##### **`revme-remote-proving.log`**

```
[2024-09-14T15:02:42Z INFO  revme_network_prove] new prover client.
[2024-09-14T15:02:42Z INFO  revme_network_prove] new prover client,ok.
[2024-09-14T15:02:42Z INFO  zkm_sdk::network::prover] calling request_proof.
[2024-09-14T15:02:46Z INFO  zkm_sdk::network::prover] calling wait_proof, proof_id=627d1511-38b9-4ccb-9372-574b435b3a80
[2024-09-14T15:02:47Z INFO  zkm_sdk::network::prover] generate_proof : queuing the task.
[2024-09-14T15:03:17Z INFO  zkm_sdk::network::prover] generate_proof : proving the task.
[2024-09-14T15:03:47Z INFO  zkm_sdk::network::prover] generate_proof : proving the task.
[2024-09-14T15:04:17Z INFO  zkm_sdk::network::prover] generate_proof : proving the task.
[2024-09-14T15:04:48Z INFO  zkm_sdk::network::prover] generate_proof : proving the task.
[2024-09-14T15:05:18Z INFO  zkm_sdk::network::prover] generate_proof : proving the task.
[2024-09-14T15:05:48Z INFO  zkm_sdk::network::prover] generate_proof : proving the task.
[2024-09-14T15:06:18Z INFO  zkm_sdk::network::prover] generate_proof : proving the task.
[2024-09-14T15:06:49Z INFO  zkm_sdk::network::prover] generate_proof : proving the task.
[2024-09-14T15:07:19Z INFO  zkm_sdk::network::prover] generate_proof : proving the task.
[2024-09-14T15:07:49Z INFO  zkm_sdk::network::prover] generate_proof : proving the task.
[2024-09-14T15:08:19Z INFO  zkm_sdk::network::prover] generate_proof : proving the task.
[2024-09-14T15:08:50Z INFO  zkm_sdk::network::prover] generate_proof : proving the task.
[2024-09-14T15:09:20Z INFO  zkm_sdk::network::prover] generate_proof : proving the task.
[2024-09-14T15:09:50Z INFO  zkm_sdk::network::prover] generate_proof : proving the task.
[2024-09-14T15:10:20Z INFO  zkm_sdk::network::prover] generate_proof : proving the task.
[2024-09-14T15:10:51Z INFO  zkm_sdk::network::prover] generate_proof : proving the task.
[2024-09-14T15:11:21Z INFO  zkm_sdk::network::prover] generate_proof : aggregating the proof.
[2024-09-14T15:11:51Z INFO  zkm_sdk::network::prover] generate_proof : aggregating the proof.
[2024-09-14T15:12:24Z INFO  revme_network_prove] Generating proof successfully .The proof file and verifier contract are in the path /tmp/zkm.
[2024-09-14T15:12:24Z INFO  revme_network_prove] Elapsed time: 581 secs
```

The proof and contract file will be in the `OUTPUT_DIR`

#### 4. Deploy the Verifier Contract

Copy the `snark_proof_with_public_inputs.json` and `verifier.sol` generated in the previous step to the contracts directory

```sh
cd zkm-project-template
export OUTPUT_DIR=/tmp/zkm  #according your setting in the run shell.
cp $OUTPUT_DIR/snark_proof_with_public_inputs.json contracts/verifier/
cp $OUTPUT_DIR/verifier.sol contracts/src/
```

If your system does not has Foundry, please install it:

```sh
curl -L https://foundry.paradigm.xyz | bash
```

Next, deploy the contract as detailed in [this](contracts/README.md) guide
