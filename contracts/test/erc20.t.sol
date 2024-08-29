// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.13;

import {Test, console} from "forge-std/Test.sol";
import {stdJson} from "forge-std/StdJson.sol";
import {Verifier} from "../src/verifier.sol";
import {StdUtils} from  "forge-std/StdUtils.sol";

struct X {
	bytes A0 ;
	bytes A1 ;
}

struct Bs {
	X x;
	X y;
}

struct Ar {
	bytes X ;
	bytes Y ;
}

struct Commitment {
	bytes X ;
	bytes Y ;
}

struct Proof {
	Ar  ar;
	Ar  krs;
	Bs  bs;
	Commitment[] commitments;
}

struct ProofPublicData{
    Proof proof;
    bytes[] publicWitness;
}

////////

struct PairingG1Point {
   uint256 X;
   uint256 Y;
}

struct PairingG2Point {
   uint256 [2]X;
   uint256 [2]Y;
}

struct VerifierProof {
	PairingG1Point A ;
	PairingG2Point B ;
	PairingG1Point C ;
}


contract VerifierTest is Test {
    using stdJson for string;

    //address verifier;
    Verifier public verifier;

    function loadProof() public view returns (ProofPublicData memory) {
        string memory root = vm.projectRoot();
        string memory path = string.concat(root, "/verifier/snark_proof_with_public_inputs.json");
        string memory json = vm.readFile(path);
        bytes memory jsonBytes = json.parseRaw(".");
        return abi.decode(jsonBytes, (ProofPublicData));
    }

  

    function test_ValidProof() public {
        ProofPublicData memory proof = loadProof();
        uint256  [65] memory input;
        for (uint256 i = 0; i < proof.publicWitness.length; i++ ){
		    input[i]= StdUtils.bytesToUint(proof.publicWitness[i]);
	    }

        VerifierProof memory verifierProof;

        verifierProof.A.X = StdUtils.bytesToUint(proof.proof.ar.X);
        verifierProof.A.Y = StdUtils.bytesToUint(proof.proof.ar.Y);

        verifierProof.B.X[0] = StdUtils.bytesToUint(proof.proof.bs.x.A0);
        verifierProof.B.X[1] = StdUtils.bytesToUint(proof.proof.bs.x.A1);

        verifierProof.B.Y[0] = StdUtils.bytesToUint(proof.proof.bs.y.A0);
        verifierProof.B.Y[1] = StdUtils.bytesToUint(proof.proof.bs.y.A1);

        verifierProof.C.X = StdUtils.bytesToUint(proof.proof.krs.X);
        verifierProof.C.Y = StdUtils.bytesToUint(proof.proof.krs.Y);

        uint256  [2] memory proofCommitment;
        proofCommitment[0] = proof.proof.Commitments[0].X;
        proofCommitment[1] = proof.proof.Commitments[0].Y;

        bool ret ;
        ret = verifier.verifyTx(verifierProof, input, proofCommitment);

         assert(ret == true);
    }
}
