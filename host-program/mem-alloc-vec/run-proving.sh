if [ $# -lt 1 ]; then
    echo "usage: ./run_proving local or network"
    exit 1
fi

set -e
program="mem-alloc-vec"
BASEDIR=$(cd $(dirname $0); pwd)
export LD_LIBRARY_PATH=$BASEDIR/../../sdk/src/local/libsnark:$LD_LIBRARY_PATH  ##Modify it according your template
export RUST_LOG=info
export SEG_SIZE=65536
export ARGS="711e9609339e92b03ddc0a211827dba421f38f9ed8b9d806e1ffdd8c15ffa03d world!"
export ELF_PATH=${BASEDIR}/../../guest-program/$program/target/mips-unknown-linux-musl/release/$program
export JSON_PATH=${BASEDIR}/../test-vectors/test.json
export PROOF_RESULTS_PATH=${BASEDIR}/../../contracts
export EXECUTE_ONLY=false
export VERIFYING_KEY_PATH=${BASEDIR}/../test-vectors/input
export SETUP_FLAG=false

##network proving
export CA_CERT_PATH=${BASEDIR}/../tool/ca.pem
export PRIVATE_KEY=   ##The private key corresponding to the public key when registering in the https://www.zkm.io/apply
export ENDPOINT=https://152.32.186.45:20002    ##the test entry of zkm proof network

echo "Compile guest-program ${program}"
if [[ "$program" =~ .*go$ ]];then
    cd $BASEDIR/../guest-program/$program
    GOOS=linux GOARCH=mips GOMIPS=softfloat go build -o $program
    export ELF_PATH=${BASEDIR}/../../guest-program/$program/$program
else
    cd $BASEDIR/../../guest-program/$program
    cargo build -r --target=mips-unknown-linux-musl
fi
cd -


echo "SEG_SIZE:$SEG_SIZE"
echo "BASEDIR:$BASEDIR"

nohup $BASEDIR/../../target/release/$program $1 >./$program-$1-proving.log 2>&1 &
echo "Check out the log by tail -f $program-$1-proving.log"
