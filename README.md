# Host Program Examples

## 0. Build guest program ELF

Please refer to : guest-program/README.md

## 1. Generate plonky2 proof

> [!NOTE]
> If the program proves succussfully, it will generate three result files in the directory verifier/data/test_circuit/.(common_circuit_data.json  proof_with_public_inputs.json  verifier_only_circuit_data.json)  

### Prove Go  program

* Run the host program 

```
RUST_LOG=info  SEG_OUTPUT=/tmp/zkm SEG_SIZE=262144 cargo run --release add-go-prove 
```

### Prove the Rust program 

* Run the host program

```
RUST_LOG=info   SEG_OUTPUT=/tmp/zkm SEG_SIZE=262144 cargo run --release revme-prove
```

## 2. Convert plonky2 proof to groth16 proof

Copy the three files generated in the previous step to the testdata/mips directory. 

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

