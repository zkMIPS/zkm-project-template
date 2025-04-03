set -e
if [ $# -lt 1 ]; then
    echo "usage: ./run_proving  revme or sha2-rust or sha2-go or mem-alloc-vec"
    exit 1
fi

program=$1
BASEDIR=$(cd $(dirname $0); pwd)
export ZKM_PROVER=${ZKM_PROVER-"local"}
#export ZKM_PROVER=network
export RUST_LOG=${RUST_LOG-info}
export SEG_SIZE=${SEG_SIZE-262144}
export ARGS="711e9609339e92b03ddc0a211827dba421f38f9ed8b9d806e1ffdd8c15ffa03d world!"
export JSON_PATH=${BASEDIR}/test-vectors/test.json
export PROOF_RESULTS_PATH=${BASEDIR}/../contracts
export EXECUTE_ONLY=false
export KEY_PATH=${BASEDIR}/../keys
export SNARK_SETUP=${SNARK_SETUP-false}

##network proving
export CA_CERT_PATH=${BASEDIR}/tool/ca.pem
export PROOF_NETWORK_PRVKEY=   ##The private key corresponding to the public key when registering in the https://www.zkm.io/apply
export ENDPOINT=https://152.32.186.45:20002    ##the test entry of zkm proof network

echo "Compile guest-program ${program}"
if [[ "$program" =~ .*go$ ]];then
    cd $BASEDIR/../guest-program/$program
    GOOS=linux GOARCH=mips GOMIPS=softfloat go build -o $program
    cd -
fi

if [ "$program" == "sha2-rust" ];then
    export SEG_SIZE=65536
elif [ "$program" == "mem-alloc-vec" ];then
     export SEG_SIZE=65536
elif [ "$program" == "sha2-composition" ];then
     export SEG_SIZE=16384
fi

echo "SEG_SIZE:$SEG_SIZE"
echo "BASEDIR:$BASEDIR"
echo "ZKM_PROVER:$ZKM_PROVER"

$BASEDIR/../target/release/$program

echo "Check out the log: tail -f $program-$ZKM_PROVER-proving.log"
