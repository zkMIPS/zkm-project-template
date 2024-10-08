set -e
program=$1
BASEDIR=$(cd $(dirname $0); pwd)
export LD_LIBRARY_PATH=$BASEDIR/../sdk/src/local/libsnark:$LD_LIBRARY_PATH  ##Modify it according your template
export ZKM_PROVER=local
export RUST_LOG=info
export SEG_SIZE=262144
export ARGS="711e9609339e92b03ddc0a211827dba421f38f9ed8b9d806e1ffdd8c15ffa03d world!"
export ELF_PATH=${BASEDIR}/../guest-program/$program/target/mips-unknown-linux-musl/release/$program

echo "Compile guest-program ${program}"
if [[ "$program" =~ .*go$ ]];then
    cd $BASEDIR/../guest-program/$program
    GOOS=linux GOARCH=mips GOMIPS=softfloat go build -o $program
else
    cd $BASEDIR/../guest-program/$program
    cargo build -r --target=mips-unknown-linux-musl
fi
cd -

echo "BASEDIR:$BASEDIR"

nohup ../target/release/zkm-prove $program >./$program-local-proving.log 2>&1 &
echo "Check out the log by tail -f $program-local-proving.log"
