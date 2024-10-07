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
