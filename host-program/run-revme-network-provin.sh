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