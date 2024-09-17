# ZKM Project Template

This is a template for creating an end-to-end ZKM project which can generate the EVM-Compatible proof and the on chain verification contract.

There are two ways to prove the guest program: 
* Use your local machine
* Use ZKM proof network 

## Local Proving Requirements
* Hardware: X86_64 CPU, 32 cores, 40G memory

* OS: Ubuntu22

* Rust: 1.81.0-nightly
  
* Go : 1.22.1
  
* Set up a local node for some blockchain(eg, sepolia)
  
  
## Network Proving Requirements
* Hardware: X86_64 CPU, 8 cores, 8G memory

* OS: Ubuntu22

* Rust: 1.81.0-nightly
  
* Go : 1.22.1
  
* CA certificate:  ca.pem, ca.key
  
* Register in the https://www.zkm.io/apply (Let your public key be in the whitelist)
  
* Set up a local node for some blockchain(eg, sepolia)


## Running the project

### 0. Download the repo

```
git clone https://github.com/zkMIPS/zkm-project-template.git
```

### 1. Build the guest program ELF

Please refer to : guest-program/README.md

### 2. Build the host program

```
cd zkm-project-template/sdk/src/local/libsnark/
./compile.sh      ##compile snark libary

cd zkm-project-template
cargo build --release
```
If it executes successfully,  it will generate two binary files in target/release : add-go-prove ,revme-prove

> [!NOTE]
> The host program executes local proving when the environmental variable ZKM_PROVER is set to "local" and performs network proving when ZKM_PROVER is set to "network".

### 3. Generate groth16 proof and verifier contract 

### (1) add-go

This program takes struct Data  as public input .

* local proving
  
```
$cd zkm-project-template/host-program
$cat run-add-go-local-proving.sh

export LD_LIBRARY_PATH=/mnt/data/zkm-project-template/sdk/src/local/libsnark:$LD_LIBRARY_PATH  ##Modify it according your template 
export ZKM_PROVER=local
export RUST_LOG=info
export SEG_SIZE=262144
export ELF_PATH=../guest-program/mips-elf/zkm-mips-elf-add-go ##If you using your own mips ELF, please modify the path
export OUTPUT_DIR=/tmp/zkm 

nohup ../target/release/add-go-prove  >./add-go-local-proving.log 2>&1 &
```
Excute the host program.
```
cd zkm-project-template/host-program
mkdir /tmp/zkm    ##Ensure that OUTPUT_DIR exists
./run-add-go-local-proving.sh
```
If successful, it will output a similar message

```
$cat add-go-local-proving.log

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
The proof and contract file will be in the OUTPUT_DIR.(snark_proof_with_public_inputs.json and verifier.sol)

* network proving

```
$cd zkm-project-template/host-program
$cat run-add-go-network-proving.sh

export CA_CERT_PATH=host-program/tool/ca.pem  #If you use your own CA, you should modify the path.
export PRIVATE_KEY=xxxxxx   ##The private key corresponding to the public key when registering in the https://www.zkm.io/apply
export LD_LIBRARY_PATH=/mnt/data/zkm-project-template/sdk/src/local/libsnark:$LD_LIBRARY_PATH  ##Modify it according your template
export ZKM_PROVER=network
export RUST_LOG=info
export SEG_SIZE=262144
export ENDPOINT=https://152.32.186.45:20002    ##the test entry of zkm proof network
export ELF_PATH=../guest-program/mips-elf/zkm-mips-elf-add-go
export OUTPUT_DIR=/tmp/zkm 

nohup ../target/release/add-go-prove  >./add-go-network-proving.log 2>&1 &
```

Excute the host program.

```
cd zkm-project-template/host-program
mkdir /tmp/zkm    ##Ensure that OUTPUT_DIR exists
./run-add-go-network-proving.sh
```
If successful, it will output a similar message.

```
$cat add-go-network-proving.log
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

The proof and contract file will be in the OUTPUT_DIR.(snark_proof_with_public_inputs.json and verifier.sol)


### (2) revme

This program  takes a block of data as public input.

* Generate the public input about a specific block
  
> [!NOTE]
> The local node is the GOAT test chain in the following example.

```
cd ~
git clone https://github.com/zkMIPS/revme
cd  revme
RPC_URL=http://localhost:8545 CHAIN_ID=1337 BLOCK_NO=244 RUST_LOG=debug SUITE_JSON_PATH=./test-vectors/244.json cargo run --example process
```
If successfully,  it will generate the 244.json in the path test-vectors.

```
cp test-vectors/244.json   zkm-project-template/host-program/test-vectors/
```

* local proving
  
```
$cd zkm-project-template/host-program
$cat run-revme-local-proving.sh

export LD_LIBRARY_PATH=/mnt/data/zkm-project-template/sdk/src/local/libsnark:$LD_LIBRARY_PATH  ##Modify it according your template 
export ZKM_PROVER=local
export RUST_LOG=info
export SEG_SIZE=262144
export ELF_PATH=../guest-program/mips-elf/zkm-mips-elf-revme-rust  ##If you using your own mips ELF, please modify the path
export PUBLIC_INPUT_PATH=host-program/test-vectors/244.json    
export OUTPUT_DIR=/tmp/zkm

nohup ../target/release/revme-prove  >./revme-local-proving.log 2>&1 &
```
Excute the host program.
```
cd zkm-project-template/host-program
mkdir /tmp/zkm    ##Ensure that OUTPUT_DIR exists
./run-revme-local-proving.sh
```
If successful, it will output a similar message.

```
$cat local-revme-proving.log

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
The proof and contract file will be in the OUTPUT_DIR.(snark_proof_with_public_inputs.json and verifier.sol)

* network proing
  
> [!NOTE]
> The proving network may sometimes experience high traffic, causing proof tasks to be queued for hours.

> The proving task requires several stages: queuing, splitting, proving, aggregating and finalizing. Each stage involves a varying duration.

```
$cd zkm-project-template/host-program
$ cat run-revme-network-provin.sh

export CA_CERT_PATH=host-program/tool/ca.pem
export PRIVATE_KEY=xxxxxx   ##The private key corresponding to the public key when registering in the https://www.zkm.io/apply
export LD_LIBRARY_PATH=/mnt/data/zkm-project-template/sdk/src/local/libsnark:$LD_LIBRARY_PATH  ##Modify it according your template
export ZKM_PROVER=network
export RUST_LOG=info
export ENDPOINT=https://152.32.186.45:20002    ##the test entry of zkm proving network
export SEG_SIZE=262144
export ELF_PATH=../guest-program/mips-elf/zkm-mips-elf-revme-rust
export PUBLIC_INPUT_PATH=host-program/test-vectors/244.json
export OUTPUT_DIR=/tmp/zkm

nohup ../target/release/revme-network-prove  >./revme-network_proving.log 2>&1 &
```

Excute the host program.
```
cd zkm-project-template/host-program
mkdir /tmp/zkm    ##Ensure that OUTPUT_DIR exists
./run-revme-network-provin.sh
```
If successful, it will output a similar message.

```
$cat revme-network_proving.log

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
The proof and contract file will be in the OUTPUT_DIR.(snark_proof_with_public_inputs.json and verifier.sol)

#### 4. Deploy Verifier Contract.

Copy the snark_proof_with_public_inputs.json and verifier.sol generated in the previous step to the contracts directory.

```
cd zkm-project-template
export OUTPUT_DIR=/tmp/zkm  ##according your setting in the run shell.
cp $OUTPUT_DIR/snark_proof_with_public_inputs.json  contracts/verifier/
cp $OUTPUT_DIR/verifier.sol contracts/src/
```

If your system does not has  Foundry,please install it.

```
curl -L https://foundry.paradigm.xyz | bash
```

Next, deploy the contract as detailed in the contracts/README.md.




