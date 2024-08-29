// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.13;

import {Test, console} from "forge-std/Test.sol";
import {stdJson} from "forge-std/StdJson.sol";
import {Verifier} from "../src/verifier.sol";


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
        for (uint256 i = 0; i < proof.PublicWitness.length; i++ ){
		    input[i]= proof.PublicWitness[i];
	    }

        VerifierProof memory verifierProof;

        verifierProof.A.X = proof.Proof.Ar.X;
        verifierProof.A.Y = proof.Proof.Ar.Y;

        verifierProof.B.X[0] = proof.Proof.Bs.X.A0;
        verifierProof.B.X[1] = proof.Proof.Bs.X.A1;

        verifierProof.B.Y[0] = proof.Proof.Bs.Y.A0;
        verifierProof.B.Y[1] = proof.Proof.Bs.Y.A1;

        verifierProof.C.X = proof.Proof.Krs.X;
        verifierProof.C.Y = proof.Proof.Krs.Y;

        uint256 memory [2]proofCommitment;
        proofCommitment[0] = proof.Proof.Commitments[0].X;
        proofCommitment[1] = proof.Proof.Commitments[0].Y;

        bool ret ;
        ret = verifier.verifyTx(verifierProof, input, proofCommitment);

         assert(ret == true);
    }
}
