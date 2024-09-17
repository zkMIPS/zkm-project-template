export CA_CERT_PATH=host-program/tool/ca.pem
export PRIVATE_KEY=xxxxxx   ##The private key corresponding to the public key when registering in the https://www.zkm.io/apply
export LD_LIBRARY_PATH=/mnt/data/gavin/zkm-project-template/sdk/src/local/libsnark:$LD_LIBRARY_PATH
export ZKM_PROVER=network
export RUST_LOG=info
export SEG_SIZE=262144
export ENDPOINT=https://152.32.186.45:20002    ##the test entry of zkm proof network
export ELF_PATH=../guest-program/mips-elf/zkm-mips-elf-add-go
export OUTPUT_DIR=/tmp/zkm 

nohup ../target/release/add-go-prove  >./add-go-network-proving.log 2>&1 &