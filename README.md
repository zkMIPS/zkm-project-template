# ZKM Project Template

This is a template for creating an end-to-end ZKM project which can generate the EVM-Compatible proof and the on chain verification contract.

There are two ways to prove the guest program: 
1. Use your local machine
2. Use ZKM Proving network 

## Local Proving

### Requirements
* Hardware: X86 CPU, 32 cores, 32G memory

* OS: Ubuntu22

* Rust:
  
* Go :
  

### 0. Build guest program ELF

Please refer to : guest-program/README.md

### 1. Generate plonky2 proof

> [!NOTE]
> If the program excutes succussfully, it will generate three  files in the directory verifier/data/test_circuit/.(common_circuit_data.json  proof_with_public_inputs.json  verifier_only_circuit_data.json)  

### Go program

* Run the host program 

```
RUST_LOG=info  SEG_OUTPUT=/tmp/zkm SEG_SIZE=262144 cargo run --release add-go-prove 
```

### Rust program 

* Run the host program

```
RUST_LOG=info   SEG_OUTPUT=/tmp/zkm SEG_SIZE=262144 cargo run --release revme-prove
```

## 2. Convert plonky2 proof to groth16 proof

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

## 3. Generate the on chain verification contract.

```
./gnark_sol_caller generate --outputDir contracts/src
```

## 4. Deploy Verifier Contract.

If your system does not has  Foundry,please install it.

```
curl -L https://foundry.paradigm.xyz | bash
```

Then, deploy the contract  refering to contracts/README.md


> [!NOTE]
> If you use ZKM proving network, the steps should be ....(TBD)

