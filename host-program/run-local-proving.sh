program=$1
BASEDIR=$(cd $(dirname $0); pwd)
export LD_LIBRARY_PATH=$BASEDIR/../sdk/src/local/libsnark:$LD_LIBRARY_PATH  ##Modify it according your template
export ZKM_PROVER=local
export RUST_LOG=info
export SEG_SIZE=262144
export ELF_PATH=${BASEDIR}/../guest-program/$program/target/mips-unknown-linux-musl/release/$program
#export PUBLIC_INPUT_PATH=host-program/test-vectors/244.json
#export OUTPUT_DIR=/tmp/$program

#nohup ../target/release/local-prove  >./local-proving.log 2>&1 &
../target/release/local-prove
