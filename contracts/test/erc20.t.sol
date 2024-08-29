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

    function bytesToUint(bytes memory b) internal pure returns (uint256) {
        require(b.length == 32, "Invalid input length");
        uint256 result;
        for (uint256 i = 0; i < 32; i++) {
            result |= uint256(uint8(b[i])) >> (i * 8);
        }
        return result;
    }

    function test_ValidProof() public {
        ProofPublicData memory proof = loadProof();
        uint256  [65] memory input;
        for (uint256 i = 0; i < proof.publicWitness.length; i++ ){
		    input[i]= bytesToUint(proof.publicWitness[i]);
	    }

        VerifierProof memory verifierProof;

        verifierProof.A.X = proof.proof.ar.X;
        verifierProof.A.Y = proof.proof.ar.Y;

        verifierProof.B.X[0] = proof.proof.bs.X.A0;
        verifierProof.B.X[1] = proof.proof.bs.X.A1;

        verifierProof.B.Y[0] = proof.proof.bs.Y.A0;
        verifierProof.B.Y[1] = proof.proof.bs.Y.A1;

        verifierProof.C.X = proof.proof.krs.X;
        verifierProof.C.Y = proof.proof.krs.Y;

        uint256  [2] memory proofCommitment;
        proofCommitment[0] = proof.proof.Commitments[0].X;
        proofCommitment[1] = proof.proof.Commitments[0].Y;

        bool ret ;
        ret = verifier.verifyTx(verifierProof, input, proofCommitment);

         assert(ret == true);
    }
}
