if [ $# -lt 1 ]; then
    echo "usage: ./run_local_proving sha2-go [or sha2-rust or mem-alloc-vec or revme]"
    exit 1
fi

set -e
program=$1
BASEDIR=$(cd $(dirname $0); pwd)
export LD_LIBRARY_PATH=$BASEDIR/../sdk/src/local/libsnark:$LD_LIBRARY_PATH  ##Modify it according your template
export ZKM_PROVER=local
export RUST_LOG=info
export SEG_SIZE=262144
export ARGS="711e9609339e92b03ddc0a211827dba421f38f9ed8b9d806e1ffdd8c15ffa03d world!"
export ELF_PATH=${BASEDIR}/../guest-program/$program/target/mips-unknown-linux-musl/release/$program
export JSON_PATH=${BASEDIR}/test-vectors/test.json
export EXECUTE_ONLY=false

echo "Compile guest-program ${program}"
if [[ "$program" =~ .*go$ ]];then
    cd $BASEDIR/../guest-program/$program
    GOOS=linux GOARCH=mips GOMIPS=softfloat go build -o $program
    export ELF_PATH=${BASEDIR}/../guest-program/$program/$program
else
    cd $BASEDIR/../guest-program/$program
    cargo build -r --target=mips-unknown-linux-musl
fi
cd -

if [ "$program" == "sha2-rust" ];then
    export SEG_SIZE=65536
elif [ "$program" == "mem-alloc-vec" ];then
     export SEG_SIZE=65536
fi
echo "SEG_SIZE:$SEG_SIZE"
echo "BASEDIR:$BASEDIR"

nohup ../target/release/zkm-prove $program >./$program-local-proving.log 2>&1 &
echo "Check out the log by tail -f $program-local-proving.log"
