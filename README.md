# ZKM Project Template

This is a template for creating an end-to-end ZKM project which can generate the EVM-Compatible proof and the on chain verification contract.

There are two ways to prove the guest program: 
* Use your local machine
* Use ZKM proof network 

## Local Proving Requirements
* Hardware: X86 CPU, 32 cores, 64G memory

* OS: Ubuntu22

* Rust: 1.81.0-nightly
  
* Go : 1.22.1
  
* Set up a local node for some blockchain(eg, sepolia)
  
  
## Network Proving Requirements
* CA certificate:  ca.pem, ca.key
  
* Register in the https://www.zkm.io/apply (Let your public key be in the whitelist)
  
* Set up a local node for some blockchain(eg, sepolia)


## Running the project

### 0. Download the repo

```
git clone https://github.com/zkMIPS/zkm-project-template.git
```

### 1. Build guest program ELF

Please refer to : guest-program/README.md

### 2. Generate the public input for some block to be proving in some blockchain
> [!NOTE]
> The local node is the GOAT test chain in the following example.

```
git clone https://github.com/zkMIPS/revme
cd  revme
RPC_URL=http://localhost:8545 CHAIN_ID=1337 BLOCK_NO=244 RUST_LOG=debug SUITE_JSON_PATH=./test-vectors/244.json cargo run --example process
```
If it executes successfully,  it will generate the 244.json in the director test-vectors.

```
cp test-vectors/244.json   zkm-project-template/host-program/test-vectors/
```

### 3. Compile the host program

```
cd zkm-project-template/sdk/src/local/libsnark/
./compile.sh      ##compile snark libary

cd zkm-project-template
cargo build --release
```
If it executes successfully,  it will generate two binary files in target/release : add-go-prove ,revme-prove

> [!NOTE]
> The host program executes local proving when the environmental variable ZKM_PROVER is set to "local" and performs network proving when ZKM_PROVER is set to "network".

### 4. Generate groth16 proof and verifier contract

#### Local proving 

* Set the Environment  parameters. 
  
```
cd zkm-project-template

export RUST_LOG=info   
export SEG_SIZE=131072
export ELF_PATH=guest-program/mips-elf/zkm-mips-elf-revme-rust 
export PUBLIC_INPUT_PATH=host-program/test-vectors/244.json
export ZKM_PROVER=local
export OUTPUT_DIR=/tmp/zkm                 ##Setting the path for saving the proof and contract
```

* proving go program

```
mkdir -p /tmp/zkm
git clone https://github.com/zkMIPS/zkm-project-template.git
cd zkm-project-template
RUST_LOG=info  SEG_OUTPUT=/tmp/zkm SEG_SIZE=262144 cargo run --release --bin add-go-prove 
```

If the memory is insufficient, please reduce the SEG_SIZE to 131072 .

* proving rust program 

```
cd zkm-project-template
RUST_LOG=info   SEG_OUTPUT=/tmp/zkm   SEG_SIZE=262144 cargo run --release --bin revme-prove
```
If the memory is insufficient, please reduce the SEG_SIZE to 131072 .

#### Network proving 

* Config your CA certificate

Put the ca.key and  ca.pem to some directory , such as , host-program/tool/ .

If you don't have a CA certificate, you can use the ca.key and  ca.pem in the  zkm-project-template/host-program/tool/.

* Set the Environment  parameters. 
  
```
cd zkm-project-template
export CA_CERT_PATH=host-program/tool/ca.pem   
export  PRIVATE_KEY=xxxxxxxxxx   ## The private key corresponding to the public key when registering in the https://www.zkm.io/apply

export RUST_LOG=info
export ENDPOINT=https://152.32.186.45:20002    
export SEG_SIZE=131072
export ELF_PATH=guest-program/mips-elf/zkm-mips-elf-revme-rust
export PUBLIC_INPUT_PATH=host-program/test-vectors/244.json
export ZKM_PROVER=network
export OUTPUT_DIR=/tmp/zkm                 ##Setting the path for saving the proof and contract
```

* Run the host program. 
  
> [!NOTE]
> The proving network may sometimes experience high traffic, causing proof tasks to be queued for hours; therefore, it is advisable to run the client in the background (or utilize a screen session).
> The proving task requires several stages: queuing, splitting, proving, aggregating and finalizing. Each stage involves a varying duration.


```
 cd zkm-project-template
 cargo build --release
 nohup ./target/release/revme-network-prove  >./network_proving.log 2>&1 &
```

If it executes successfully,  it will output the similar message:
```
tail -f network_proving.log

[2024-09-11T02:33:27Z INFO  revme_network_prove] new prover client.
[2024-09-11T02:33:28Z INFO  revme_network_prove] new prover client,ok.
[2024-09-11T02:33:28Z INFO  zkm_sdk::network::prover] calling request_proof.
[2024-09-11T02:33:45Z INFO  zkm_sdk::network::prover] calling wait_proof, proof_id=cbac84b8-d5bc-4d39-a7f2-be8ffccd91bc
[2024-09-11T02:33:45Z INFO  zkm_sdk::network::prover] generate_proof : queuing the task.
[2024-09-11T02:34:16Z INFO  zkm_sdk::network::prover] generate_proof : splitting the task.
[2024-09-11T02:34:46Z INFO  zkm_sdk::network::prover] generate_proof : splitting the task.
[2024-09-11T02:35:16Z INFO  zkm_sdk::network::prover] generate_proof : splitting the task.
[2024-09-11T02:35:46Z INFO  zkm_sdk::network::prover] generate_proof : splitting the task.
[2024-09-11T02:36:17Z INFO  zkm_sdk::network::prover] generate_proof : splitting the task.
[2024-09-11T02:36:47Z INFO  zkm_sdk::network::prover] generate_proof : splitting the task.
[2024-09-11T02:37:17Z INFO  zkm_sdk::network::prover] generate_proof : splitting the task.
[2024-09-11T02:37:47Z INFO  zkm_sdk::network::prover] generate_proof : splitting the task.
[2024-09-11T02:38:18Z INFO  zkm_sdk::network::prover] generate_proof : splitting the task.
[2024-09-11T02:38:48Z INFO  zkm_sdk::network::prover] generate_proof : proving the task.
[2024-09-11T02:39:18Z INFO  zkm_sdk::network::prover] generate_proof : proving the task.
[2024-09-11T02:39:48Z INFO  zkm_sdk::network::prover] generate_proof : proving the task.
[2024-09-11T02:40:18Z INFO  zkm_sdk::network::prover] generate_proof : proving the task.
[2024-09-11T02:40:49Z INFO  zkm_sdk::network::prover] generate_proof : proving the task.
[2024-09-11T02:41:19Z INFO  zkm_sdk::network::prover] generate_proof : proving the task.
//...
[2024-09-11T07:22:08Z INFO  zkm_sdk::network::prover] generate_proof : proving the task.
[2024-09-11T07:22:38Z INFO  zkm_sdk::network::prover] generate_proof : aggregating the proof.
[2024-09-11T07:23:08Z INFO  zkm_sdk::network::prover] generate_proof : aggregating the proof.
[2024-09-11T07:23:38Z INFO  zkm_sdk::network::prover] generate_proof : aggregating the proof.
[2024-09-11T07:24:09Z INFO  zkm_sdk::network::prover] generate_proof : aggregating the proof.
[2024-09-11T07:24:39Z INFO  zkm_sdk::network::prover] generate_proof : aggregating the proof.
[2024-09-11T07:25:09Z INFO  zkm_sdk::network::prover] generate_proof : aggregating the proof.
[2024-09-11T07:25:39Z INFO  zkm_sdk::network::prover] generate_proof : aggregating the proof.
[2024-09-11T07:28:41Z INFO  zkm_sdk::network::prover] generate_proof : aggregating the proof.
[2024-09-11T07:29:11Z INFO  zkm_sdk::network::prover] generate_proof : aggregating the proof.
[2024-09-11T07:29:41Z INFO  zkm_sdk::network::prover] generate_proof : aggregating the proof.
[2024-09-11T07:30:11Z INFO  zkm_sdk::network::prover] generate_proof : aggregating the proof.
[2024-09-11T07:30:42Z INFO  zkm_sdk::network::prover] generate_proof : finalizing the proof.
[2024-09-11T07:31:14Z INFO  revme_network_prove] Generating proof successfully .The proof file and verifier contract are in the path /tmp/zkm.
[2024-09-11T07:31:14Z INFO  revme_network_prove] Elapsed time: 17866 secs

```
  

#### 4. Deploy Verifier Contract.

If your system does not has  Foundry,please install it.

```
curl -L https://foundry.paradigm.xyz | bash
```

Then, deploy the contract  refering to "### Deploy" in the contracts/README.md .




