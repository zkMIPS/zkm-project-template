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

There are four main ways to run this project: 
- **Execute** a program.
- Generate a **core** proof. 
- Generate a **compressed** proof.
- Generate an **EVM-compatible** proof.

### Build the Program

The program is automatically built through `./host/build.rs` whenever the host is built.

### Execute the Program

To run the guest program without generating a proof:
```sh
cd host
cargo run --release -- --execute
```

This will execute the guest program and display the output.

### Generate an zkMIPS Core Proof

To generate a zkMIPS [core proof](https://docs.zkm.io/dev/prover.html#proof-types) for your guest program:

```sh
cd host
cargo run --release -- --core
```
### Generate an zkMIPS Compressed Proof

To generate a zkMIPS [compressed proof](https://docs.zkm.io/dev/prover.html#proof-types) for your guest program:

```sh
cd host
cargo run --release -- --compressed
```

### Generate an EVM-Compatible Proof
Producing a proof that’s cheap to verify on Ethereum (e.g., Groth16 or PLONK) is more computationally intensive than generating a core or compressed proof.

- To generate a Groth16 proof:
```sh
cd host
cargo run --release --bin evm -- --system groth16
```

- To generate a PLONK proof:
```sh
cargo run --release --bin evm -- --system plonk
```
These commands will also generate fixtures that can be used to test verification of zkMIPS proofs in Solidity.

>[!NOTE]
> Do not set `ZKM_PROVER=network` when generating a core, compressed or PLONK proof — the network prover only supports Groth16.


### Retrieve the Verification Key

To retrieve your `programVKey` for your on-chain contract, run the following command in `host`:

```sh
cargo run --release --bin vkey
```

## Using the Prover Network

Refer to this [document](https://docs.zkm.io/dev/prover.html#network-prover).
The proving workflow involves several stages—queuing, splitting, proving, aggregating, and finalizing—
each of which can take varying amounts of time. Before running the prover, ensure that you have set up the following environment variables in your `.env`:

```env
ZKM_PROVER=network
ZKM_PRIVATE_KEY=
CERT_PATH=
KEY_PATH=
```

### Deploy the Verifier Contract

If your system does not have Foundry, please install it:

```sh
curl -L https://foundry.paradigm.xyz | bash
```
#### Verify the EVM-Compatible Proof

```
cd  zkm-project-template/contracts
forge test
```

#### Deploy the contract

Please edit the following parameters according your aim blockchain.

```
forge script script/verifier.s.sol:VerifierScript --rpc-url https://eth-sepolia.g.alchemy.com/v2/RH793ZL_pQkZb7KttcWcTlOjPrN0BjOW --private-key df4bc5647fdb9600ceb4943d4adff3749956a8512e5707716357b13d5ee687d9
```

For more details, please refer to [this](contracts/README.md) guide.
