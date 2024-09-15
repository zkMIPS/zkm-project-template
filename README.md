# ZKM Project Template

This is a template for creating an end-to-end ZKM project which can generate the EVM-Compatible proof and the on chain verification contract.

There are two ways to prove the guest program: 
* Use your local machine
* Use ZKM proof network 

## Local Proving Requirements
* Hardware: X86_64 CPU, 32 cores, 64G memory

* OS: Ubuntu22

* Rust: 1.81.0-nightly
  
* Go : 1.22.1
  
* Set up a local node for some blockchain(eg, sepolia)
  
  
## Network Proving Requirements
* Hardware: X86_64 CPU, 8 cores, 16G memory

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

### 1. Build guest program ELF

Please refer to : guest-program/README.md

### 2. Compile the host program

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

#### add-go

* local proving
  
```
$cd zkm-project-template
$cat run-add-go-local-proving.sh
export LD_LIBRARY_PATH=/mnt/data/zkm-project-template/sdk/src/local/libsnark:$LD_LIBRARY_PATH  ##Modify it according your template 
export ZKM_PROVER=local
export RUST_LOG=info
export SEG_SIZE=262144
export ELF_PATH=guest-program/mips-elf/zkm-mips-elf-add-go ##If you using your own mips ELF, please modify the path
export OUTPUT_DIR=/tmp/zkm 

nohup ./target/release/add-go-prove  >./add-go-local-proving.log 2>&1 &
```
Excute the host program.
```
cd zkm-project-template
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
panic: runtime error: slice bounds out of range [48:47]

goroutine 1 [running]:
github.com/zkMIPS/zkm/go-runtime/zkm_runtime.deserializeData({0x42e000, 0x4b, 0x4c}, {0xd6400, 0x422710, 0x197}, 0x28)
        /mnt/data/gavin/zkm/go-runtime/zkm_runtime/deserialize.go:88 +0x116c
github.com/zkMIPS/zkm/go-runtime/zkm_runtime.deserializeData({0x42e000, 0x4b, 0x4c}, {0xe0da0, 0x434ee8, 0x199}, 0x0)
        /mnt/data/gavin/zkm/go-runtime/zkm_runtime/deserialize.go:122 +0x10a0
github.com/zkMIPS/zkm/go-runtime/zkm_runtime.DeserializeData({0x42e000, 0x4b, 0x4c}, {0xd4600, 0x4226e8})
        /mnt/data/gavin/zkm/go-runtime/zkm_runtime/deserialize.go:28 +0x1d4
github.com/zkMIPS/zkm/go-runtime/zkm_runtime.Read[...]()
        /mnt/data/gavin/zkm/go-runtime/zkm_runtime/runtime.go:17 +0x11c
main.main()
        /mnt/data/gavin/zkm/prover/examples/add-go/add.go:30 +0x48
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

> [!NOTE]
> The proving network may sometimes experience high traffic, causing proof tasks to be queued for hours; therefore, it is advisable to run the client in the background (or utilize a screen session).
> The proving task requires several stages: queuing, splitting, proving, aggregating and finalizing. Each stage involves a varying duration.

```
$cd zkm-project-template
$cat run-add-go-network-proving.sh

export CA_CERT_PATH=host-program/tool/ca.pem  #If you use your own CA, you should modify the path.
export PRIVATE_KEY=xxxxxx   ##The private key corresponding to the public key when registering in the https://www.zkm.io/apply
export LD_LIBRARY_PATH=/mnt/data/gavin/zkm-project-template/sdk/src/local/libsnark:$LD_LIBRARY_PATH
export ZKM_PROVER=network
export RUST_LOG=info
export SEG_SIZE=262144
export ENDPOINT=https://152.32.186.45:20002    ##the test entry of zkm proof network
export ELF_PATH=guest-program/mips-elf/zkm-mips-elf-add-go
export OUTPUT_DIR=/tmp/zkm 

nohup ./target/release/add-go-prove  >./add-go-network-proving.log 2>&1 &
```

Excute the host program.

```
cd zkm-project-template
mkdir /tmp/zkm    ##Ensure that OUTPUT_DIR exists
./run-add-go-network-proving.sh
```
If successful, it will output a similar message

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


#### revme

* Generate the public input about a specific block
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

* local proving
  
  
* network proing


#### 4. Deploy Verifier Contract.

If your system does not has  Foundry,please install it.

```
curl -L https://foundry.paradigm.xyz | bash
```

Then, deploy the contract  refering to "### Deploy" in the contracts/README.md .




