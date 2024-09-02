# ZKM Project Template

This is a template for creating an end-to-end ZKM project which can generate the EVM-Compatible proof and the on chain verification contract.

There are two ways to prove the guest program: 
* Use your local machine
* Use ZKM Proving network 

## Local Proving

### Requirements
* Hardware: X86 CPU, 32 cores, 32G memory

* OS: Ubuntu22

* Rust: 1.81.0-nightly
  
* Go : 1.22.1
  
### Running the project

#### 0. Build guest program ELF

Please refer to : guest-program/README.md

#### 1. Generate plonky2 proof

> [!NOTE]
> If the program excutes succussfully, it will generate three  files in the directory verifier/data/test_circuit/.(common_circuit_data.json  proof_with_public_inputs.json  verifier_only_circuit_data.json)  

* Go program

```
mkdir -p /tmp/zkm
git clone https://github.com/zkMIPS/zkm-project-template.git
cd zkm-project-template
RUST_LOG=info  SEG_OUTPUT=/tmp/zkm SEG_SIZE=262144 cargo run --release --bin add-go-prove 
```

If the memory is insufficient, please reduce the SEG_SIZE to 131072 .

* Rust program 

```
cd zkm-project-template
RUST_LOG=info   SEG_OUTPUT=/tmp/zkm   SEG_SIZE=262144 cargo run --release --bin revme-prove
```
If the memory is insufficient, please reduce the SEG_SIZE to 131072 .


#### 2. Convert plonky2 proof to groth16 proof

Copy the  three files generated in the previous step to the testdata/mips directory. 

```
cp verifier/data/test_circuit/* testdata/mips
./benchmark
```

If benchmark excutes succussfully, it will generate a groth16 proof and  verifying.key in the directory testdata/mips/.
Then, copying the proof file and  verifying.key to contracts/verifier

```
cp testdata/mips/snark_proof_with_public_inputs.json    contracts/verifier/
cp testdata/mips/verifying.key  contracts/verifier/
```

#### 3. Generate the on chain verification contract.

```
./gnark_sol_caller generate --outputDir contracts/src
```

#### 4. Deploy Verifier Contract.

If your system does not has  Foundry,please install it.

```
curl -L https://foundry.paradigm.xyz | bash
```

Then, deploy the contract  refering to "### Deploy" in the contracts/README.md .

## Network Proving

> [!NOTE]
> The proving network is demo at present. The production version is coming soon.

### Requirements
* CA certificate:  ca.pem, ca.key
* Register in the https://www.zkm.io/apply (Let your key be in the whitelist)
* Set up a local node for some blockchain(eg, sepolia)

### Running the project

#### 0. Build guest program ELF

Please refer to : guest-program/README.md

#### 1. Config your CA certificate

Put the ca.key and  ca.pem to some directory , such as , host-program/tool/ .

If you don't have a CA certificate, you can use the ca.key and  ca.pem in the  zkm-project-template/host-program/tool/.

#### 2. Generate the public input for some block to be proving in some blockchain
> [!NOTE]
> The local node is GOAT chain in the following example.

```
git clone https://github.com/zkMIPS/revme
cd  revme
RPC_URL=http://localhost:8545 CHAIN_ID=1337 BLOCK_NO=244 RUST_LOG=debug SUITE_JSON_PATH=./test-vectors/244.json cargo run --example process
```
If it executes successfully,  it will generate the 244.json in the director test-vectors.

```
cp test-vectors/244.json   zkm-project-template/host-program/test-vectors/
```

#### 3. Generate the groth16 proof and  verifier Contract

* Set the Environment  parameters. 
  
```
cd zkm-project-template
export CA_CERT_PATH=host-program/tool/ca.pem   
export  PRIVATE_KEY=df4bc5647fdb9600ceb4943d4adff3749956a8512e5707716357b13d5ee687d9   ##For testing, No changing the key!

export RUST_LOG=info
export ENDPOINT=https://152.32.186.45:20002    ##the test entry of zkm proving network 
export SEG_SIZE=131072
export ELF_PATH=guest-program/mips-elf/zkm-mips-elf-revme-rust
export PUBLIC_INPUT_PATH=host-program/test-vectors/244.json
```

* Run the host program. 

```
  cargo run --release  --bin revme-network-prove
```

If it executes successfully,  it will output the similar message:
```
[2024-08-28T03:20:55Z INFO  stage] request: "1509d5b6-a9e3-4b2f-85b8-5739c35a1310"
[2024-08-28T03:20:58Z INFO  stage] generate_proof response: GenerateProofResponse { status: 2, error_message: "", proof_id: "1509d5b6-a9e3-4b2f-85b8-5739c35a1310", proof_url: "http://152.32.186.45:20001/1509d5b6-a9e3-4b2f-85b8-5739c35a1310/final/proof_with_public_inputs.json", stark_proof_url: "http://152.32.186.45:20001/1509d5b6-a9e3-4b2f-85b8-5739c35a1310/aggregate/proof_with_public_inputs.json", solidity_verifier_url: "http://152.32.186.45:20001/verifier.sol", output_stream: [] }
[2024-08-28T03:21:52Z INFO  stage] generate_proof success public_inputs_size: 1546, output_size: 0
[2024-08-28T03:21:52Z INFO  stage] Elapsed time: 56 secs
```

* Download the proof and contract

In the above output, we need the proof_url: "http://152.32.186.45:20001/1509d5b6-a9e3-4b2f-85b8-5739c35a1310/final/proof_with_public_inputs.json" and solidity_verifier_url: "http://152.32.186.45:20001/verifier.sol" .

```
wget http://152.32.186.45:20001/1509d5b6-a9e3-4b2f-85b8-5739c35a1310/final/proof_with_public_inputs.json
wget http://152.32.186.45:20001/verifier.sol
```
Then, move the proof and verifier.sol to contracts directory.
```
mv proof_with_public_inputs.json  contracts/verifier/
mv verifier.sol contracts/src/
```

#### 4. Deploy Verifier Contract.

If your system does not has  Foundry,please install it.

```
curl -L https://foundry.paradigm.xyz | bash
```

Then, deploy the contract  refering to contracts/README.md


