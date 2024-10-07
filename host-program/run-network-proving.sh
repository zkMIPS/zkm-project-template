program=$1
BASEDIR=$(cd $(dirname $0); pwd)
export LD_LIBRARY_PATH=$BASEDIR/../sdk/src/local/libsnark:$LD_LIBRARY_PATH  ##Modify it according your template
export CA_CERT_PATH=tool/ca.pem
export PRIVATE_KEY=xxxx   ##The private key corresponding to the public key when registering in the https://www.zkm.io/apply
export ENDPOINT=https://152.32.186.45:20002    ##the test entry of zkm proving network
export ZKM_PROVER=network
export RUST_LOG=info
export SEG_SIZE=62144
export ARGS="711e9609339e92b03ddc0a211827dba421f38f9ed8b9d806e1ffdd8c15ffa03d world!"
#export ELF_PATH=${BASEDIR}/../guest-program/$program/target/mips-unknown-linux-musl/release/$program
#export PUBLIC_INPUT_PATH=host-program/test-vectors/244.json
#export OUTPUT_DIR=/tmp/$program
echo "BASEDIR:$BASEDIR"


nohup ../target/release/zkm-prove $program >./$program-network-proving.log 2>&1 &
echo 'check out the log by tail -f $program-network-proving.log'
