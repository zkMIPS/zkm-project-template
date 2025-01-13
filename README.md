# ZKM Project Template

The Project Template allows the developer to create an end-to-end zkMIPS project and the on-chain Solidity verifier.

Two provers have been provided:

- Local Prover: Use your machine to run the prover and generate the proof by your end.
- Network Prover: Use ZKM proof network to generate the proof via our Restful API. 

## Running diagram

![image](assets/temp-run-diagram.png)

## Getting Started

First to install zkm toolchain run the following command and follow the instructions:
```sh
curl --proto '=https' --tlsv1.2 -sSf https://raw.githubusercontent.com/zkMIPS/toolchain/refs/heads/main/setup.sh | sh
source ~/.zkm-toolchain/env
```

## Template code structure

> [!NOTE]
> The SDK has a libary(libsnark) which supports local proving. If the libsnark is required, please specify the features = ["snark"] in your Cargo.toml. To disable libsnark, set the environment variable NO_USE_SNARK to true when compiling the SDK.

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

Please refer to [this](guest-program/README.md) guide.

### 2. Build the host program

```sh
cd zkm-project-template
sh sdk/src/local/libsnark/compile.sh  # compile snark library
cargo build --release                 # build host programs
```

If successfully, it will generate the binary files in `target/release`/{`sha2-rust` ,`sha2-go` ,`revme`, `mem-alloc-vec`}

> [!NOTE]
> You can run the guest program without generating a proof by setting the environmental variable `EXECUTE_ONLY` to "true".https://github.com/zkMIPS/zkm/issues/152

> You can set the `ZKM_SKIP_PROGRAM_BUILD` environment variable to `true` to skip building the guest program when use `zkm_build::build_program`.

### 3. Generate groth16 proof and verifier contract

> [!NOTE]
> 1. There is  a script program available: run_proving.sh. The script facilitate the generation of proofs on the local machine and over the proof network.

> 2. There are four guest programs(sha2-rust, sha2-go, mem-alloc-vec,revme). The following will use sha2-rust and revme as an example to demonstrate local and network proofs.

> 3. If the environmental variable `PROOF_RESULTS_PATH` is not set, the proof results file will be saved in zkm-project-template/contracts/{src, verifier}; if the environmental variable `PROOF_RESULTS_PATH` is set, after the proof is completed, the proof results file needs to be copied from from 'PROOF_RESULTS_PATH'/{src, verifier} to the corresponding zkm-project-template/contracts/{src, verifier}. 

> 4. The environment variable `VERIFYING_KEY_PATH` specifies the location of the verification key (vk). If this variable is not set to zkm-project-template/contracts/src, you should copy the  `VERIFYING_KEY_PATH`/verifier.sol to zkm-project-template/contracts/src/ after executing the host program.

> 5. The environment variable `SETUP_FLAG` is set to "true", it will generate  the proof key (pk), the verification key (vk) and the verifier contract and store them at the path indicated by `VERIFYING_KEY_PATH`.Then, the `SETUP_FLAG` should be set to "false" , next executing the host will  generate the snark proof using the same pk and vk.

> [!WARNING]
>  The environmental variable `SEG_SIZE` in the run_proving.sh affects the final proof generation. 

>  The guest program's ELF with the input is split into segments according the SEG_SIZE, based on the cycle count.

>  When generating proofs on the local machine, if the log shows "[the seg_num is:1 ]", please reduce SEG_SIZE or increase the input. If generating proofs through the proof network, SEG_SIZE must be within the range [65536, 262144]. 

### Example 1 : `sha2-rust`

This host program sends the private input pri_input = vec![5u8; 1024] and its hash (hash(pri_input)) to the guest program for verification of the hash value.

#### Local Proving

Make any edits to [`run-proving.sh`](host-program/run-proving.sh) and run the program:

```sh
cd zkm-project-template/host-program
./run-proving.sh sha2-rust
```

The result proof and contract file will be in the contracts/verifier and contracts/src respectively.

#### Network Proving

> [!NOTE]
> The proving network may sometimes experience high traffic, causing proof tasks to be queued for hours.

> The proving task requires several stages: queuing, splitting, proving, aggregating and finalizing. Each stage involves a varying duration.

Must set the `PROOF_NETWORK_PRVKEY` and `ZKM_PROVER=network` in [`run-proving.sh`](host-program/run-proving.sh) and run the program:

```sh
./run-proving.sh sha2-rust
```

The result proof and contract file will be in the contracts/verifier and contracts/src.

### 4. Deploy the Verifier Contract

If your system does not has Foundry, please install it:

```sh
curl -L https://foundry.paradigm.xyz | bash
```
#### Verify the snark proof generateing in the step 3

```
cd  zkm-project-template/contracts
forge test
```

#### Deploy the contract generateing in the step 3

Please edit the following parameters according your aim blockchain.

```
forge script script/verifier.s.sol:VerifierScript --rpc-url https://eth-sepolia.g.alchemy.com/v2/RH793ZL_pQkZb7KttcWcTlOjPrN0BjOW --private-key df4bc5647fdb9600ceb4943d4adff3749956a8512e5707716357b13d5ee687d9
```

For more details, please refer to [this](contracts/README.md) guide.

### Example 2 : `revme`

The revme guest program takes a block data as input and its running is as same as the sha2-rust. Here, the focus is on explaining how to generate block data(the revme's input).

#### Generating the public input about a specific block

> [!NOTE]
> The local node connects  ZKM test chain in the following example. You must use the Eth-Compatible local node.

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

Next, you need to edit the `JSON_PATH` variable in the [`run-proving.sh`](host-program/revme/run-proving.sh) to match the name of the  JSON file mentioned above.

Then, you can execute the run-proving.sh by following the steps outlined in `Example 1: sha2-rust`.
