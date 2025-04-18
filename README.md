# ZKM Project Template

The Project Template allows the developer to create an end-to-end zkMIPS project and the on-chain Solidity verifier.

Two provers have been provided:

- Local Prover: Use your machine to run the prover and generate the proof by your end.
- Network Prover: Use ZKM proof network to generate the proof via our Restful API. 

## Running diagram

![image](assets/temp-run-diagram.png)

## Getting Started

First to install zkMIPS toolchain run the following command and follow the instructions:
```sh
curl --proto '=https' --tlsv1.2 -sSf https://raw.githubusercontent.com/zkMIPS/toolchain/refs/heads/main/setup.sh | sh
source ~/.zkm-toolchain/env
```

## Template code structure

## Local Proving Requirements

- Hardware: X86_64 CPU, 32 cores, 16GB memory (minimum)
- OS: Linux
- Rust: 1.81.0-nightly

## Network Proving Requirements

- Hardware: X86_64 CPU, 8 cores, 8G memory
- OS: Linux
- Rust: 1.81.0-nightly
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

The program is automatically built through `host/build.rs` when the host is built.

### 2. Execute the Program

To run the guest program without generating a proof:

```sh
cd host
cargo run --release -- --execute
```

This will execute the guest program and display the output.

### 3. Generate a Proof

#### Local Proving

**Generate a zkMIPS Core Proof**

This step can be skipped if only EVM-compatible proof is needed.

To generate an zkMIPS [core proof](https://docs.zkm.io/dev/prover.html#proof-types) for your guest program:

```sh
cd host
cargo run --release -- --prove
```

**Generate an EVM-Compatible Proof**

> [!WARNING]
> You will need at least 16GB RAM to generate a Groth16 or PLONK proof.

Generating a proof that is cheap to verify on the EVM (e.g. Groth16 or PLONK) is more intensive than generating a core proof.

To generate a Groth16 proof:

```sh
cd host
cargo run --release --bin evm -- --system groth16
```

To generate a PLONK proof:

```sh
cargo run --release --bin evm -- --system plonk
```

These commands will also generate fixtures that can be used to test the verification of zkMIPS proofs
inside Solidity.

**Retrieve the Verification Key**

To retrieve your `programVKey` for your on-chain contract, run the following command in `host`:

```sh
cargo run --release --bin vkey
```

#### Network Proving

> [!NOTE]
> The proving network may sometimes experience high traffic, causing proof tasks to be queued for hours.

> The proving task requires several stages: queuing, splitting, proving, aggregating and finalizing. Each stage involves a varying duration.

Must set the `PROOF_NETWORK_PRVKEY` and `ZKM_PROVER=network` in [`run-proving.sh`](host-network/run-proving.sh) and run the program:

```sh
cd host-network
./proving.sh
```

The result proof file will be in the `host-network/output`.

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
forge script script/ZKMVerifierGroth16.s.sol:ZKMVerifierGroth16Script --rpc-url https://eth-sepolia.g.alchemy.com/v2/RH793ZL_pQkZb7KttcWcTlOjPrN0BjOW --private-key df4bc5647fdb9600ceb4943d4adff3749956a8512e5707716357b13d5ee687d9

forge script script/ZKMVerifierPlonk.s.sol:ZKMVerifierPlonkScript --rpc-url https://eth-sepolia.g.alchemy.com/v2/RH793ZL_pQkZb7KttcWcTlOjPrN0BjOW --private-key df4bc5647fdb9600ceb4943d4adff3749956a8512e5707716357b13d5ee687d9
```

For more details, please refer to [this](contracts/README.md) guide.
