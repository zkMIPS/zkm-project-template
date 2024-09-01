// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.13;

import {Test, console} from "forge-std/Test.sol";
import {stdJson} from "forge-std/StdJson.sol";
import {Verifier} from "../src/verifier.sol";
import {StdUtils} from  "forge-std/StdUtils.sol";
import {console}  from "forge-std/console.sol";

struct XX {
	bytes A0 ;
	bytes A1 ;
}

struct BS {
	XX X;
	XX Y;
}

struct AR {
	bytes X ;
	bytes Y ;
}

struct Commitment {
	bytes X ;
	bytes Y ;
}

struct PROOF {
	AR  Ar ;
	AR  Krs;
	BS  Bs;
	Commitment[] Commitments;
    Commitment   CommitmentPok;
}

struct ProofPublicData{
    PROOF Proof;
    bytes[] PublicWitness;
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

    Verifier public verifier;
    

    function loadProof() public view returns (ProofPublicData memory) {
        string memory root = vm.projectRoot();
        string memory path = string.concat(root, "/verifier/snark_proof_with_public_inputs.json");
        string memory json = vm.readFile(path);
        bytes memory jsonBytes = json.parseRaw(".");
        return abi.decode(jsonBytes, (ProofPublicData));
    }

    function setUp() public {
        ProofPublicData memory proof = loadProof();

        verifier = new Verifier();
    }
  
    function test_ValidProof() public {
        ProofPublicData memory proof = loadProof();
        uint256  [65] memory input;
        for (uint256 i = 0; i < proof.PublicWitness.length; i++ ){
		    input[i]= StdUtils.bytesToUint(proof.PublicWitness[i]);
	    }
        
         Verifier.Proof memory verifierProof;

        verifierProof.a.X = StdUtils.bytesToUint(proof.proof.Ar.X);
        verifierProof.a.Y = StdUtils.bytesToUint(proof.proof.Ar.Y);

        verifierProof.b.X[0] = StdUtils.bytesToUint(proof.proof.Bs.X.A0);
        verifierProof.b.X[1] = StdUtils.bytesToUint(proof.proof.Bs.X.A1);

        verifierProof.b.Y[0] = StdUtils.bytesToUint(proof.proof.Bs.Y.A0);
        verifierProof.b.Y[1] = StdUtils.bytesToUint(proof.proof.Bs.Y.A1);

        verifierProof.c.X = StdUtils.bytesToUint(proof.proof.Krs.X);
        verifierProof.c.Y = StdUtils.bytesToUint(proof.proof.Krs.Y);

        uint256  [2] memory proofCommitment;
        proofCommitment[0] = StdUtils.bytesToUint(proof.proof.commitments[0].X);
        proofCommitment[1] = StdUtils.bytesToUint(proof.proof.commitments[0].Y);

        bool ret ;
        ret = verifier.verifyTx(verifierProof, input, proofCommitment);

         assert(ret == true);
    }
}
