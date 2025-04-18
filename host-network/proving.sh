#!/bin/bash -e

BASEDIR=$(cd $(dirname $0); pwd)
export ZKM_PROVER=${ZKM_PROVER-"network"}
export RUST_LOG=${RUST_LOG-info}
export SEG_SIZE=${SEG_SIZE-262144}
export OUTPUT_DIR=${BASEDIR}/output
export EXECUTE_ONLY=false

## network proving
export CA_CERT_PATH=${BASEDIR}/tool/ca.pem
export CERT_PATH=${BASEDIR}/tool/cert.pem
export KEY_PATH=${BASEDIR}/tool/key.pem
## The private key corresponding to the public key when registering in the https://www.zkm.io/apply
export PROOF_NETWORK_PRVKEY=7649f495f2215e64ee1ab02359fa85d25dbeec9c084ea34ca0e3117704dd1904
export ENDPOINT=https://152.32.186.45:20002    ##the test entry of zkm proof network
export DOMAIN_NAME=stage

cargo run --release
