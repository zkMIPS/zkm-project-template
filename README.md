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
RUST_LOG=info  SEG_OUTPUT=/tmp/zkm SEG_SIZE=262144 cargo run --release add-go-prove 
```

If the memory is insufficient, please reduce the SEG_SIZE to 131072 .

* Rust program 

```
RUST_LOG=info   SEG_OUTPUT=/tmp/zkm SEG_SIZE=262144 cargo run --release revme-prove
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

Then, deploy the contract  refering to contracts/README.md

## Network Proving

> [!NOTE]
> The proving network is demo at present. The production version is coming soon.

### Requirements
* CA certificate:  ca.pem

### Running the project

#### 0. Build guest program ELF

Please refer to : guest-program/README.md

#### 1. Download the block 

The block has your transaction.
We use the following tool to download the block.

* Compile the tool. 

```
$ git clone https://github.com/zkMIPS/cannon-mips
$ cd cannon-mips && make 
```

* Config the tool. 
  
```
$ mkdir -p /tmp/cannon
$ export BASEDIR=/tmp/cannon; 
$ export NODE=https://eth-sepolia.g.alchemy.com/v2/RH793ZL_pQkZb7KttcWcTlOjPrN0BjOW 
```

* Download some block. 

```
$ minigeth/go-ethereum 13284491
```
If it excutes successfully, you will see the block infomation in the directory /tmp/cannon/0_13284491 .

#### 2. Config your CA certificate

Put the ca.key and  ca.pem to some directory , such as , host-program/tool/ .

If you don't have a CA certificate, you can generate it using the  certgen.sh in the director zkm-project-template/host-program/tool/.
```
$ cd zkm-project-template/host-program/tool/
$ chmod +x certgen.sh
$ ./certgen.sh
```



