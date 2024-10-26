// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.13;

import {Test, console} from "forge-std/Test.sol";
import {stdJson} from "forge-std/StdJson.sol";
import {Verifier} from "../src/verifier.sol";
import {StdUtils} from  "forge-std/StdUtils.sol";
import {console}  from "forge-std/console.sol";

struct XX {
	string a0 ;
	string a1 ;
}

struct BS {
	XX x;
	XX y;
}

struct AR {
	string x ;
	string y ;
}

struct Commitment {
	string x ;
	string y ;
}

struct PROOF {
	AR  ar ;
	AR  krs;
	BS  bs;
	Commitment[1] commitments;
    Commitment   commitmentPok;
}

struct ProofPublicData{
    PROOF proof;
    string[2] publicWitness;
}

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
    
    function setUp() public {
        verifier = new Verifier();
    }
  
    function test_ValidProof() public {
        string memory root = vm.projectRoot();
        string memory path = string.concat(root, "/verifier/snark_proof_with_public_inputs.json");
        string memory json = vm.readFile(path);

        ProofPublicData memory proofData;

        bytes memory ProofAr = json.parseRaw(".Proof.Ar");
        proofData.proof.ar = abi.decode(ProofAr, (AR));
        
        bytes memory ProofKrs = json.parseRaw(".Proof.Krs");
        proofData.proof.krs = abi.decode(ProofKrs, (AR));

        bytes memory ProofBs = json.parseRaw(".Proof.Bs");
        proofData.proof.bs = abi.decode(ProofBs, (BS));

        bytes memory ProofCommitments = json.parseRaw(".Proof.Commitments[0]");
        proofData.proof.commitments[0] = abi.decode(ProofCommitments, (Commitment));

        bytes memory ProofCommitment = json.parseRaw(".Proof.CommitmentPok");
        proofData.proof.commitmentPok = abi.decode(ProofCommitment, (Commitment));

        bytes memory publicWitness = json.parseRaw(".PublicWitness");
        string[] memory pubwit = abi.decode(publicWitness, ( string[]));
        
        uint  [2] memory input;
         for (uint256 i = 0; i < pubwit.length; i++ ){
            input[i]  =  vm.parseUint(pubwit[i]);
	    }

        Verifier.Proof memory verifierProof;

        verifierProof.a.X =  vm.parseUint(proofData.proof.ar.x);
        verifierProof.a.Y = vm.parseUint(proofData.proof.ar.y);

        verifierProof.b.X[0] = vm.parseUint(proofData.proof.bs.x.a0);
        verifierProof.b.X[1] = vm.parseUint(proofData.proof.bs.x.a1);

        verifierProof.b.Y[0] = vm.parseUint(proofData.proof.bs.y.a0);
        verifierProof.b.Y[1] = vm.parseUint(proofData.proof.bs.y.a1);

        verifierProof.c.X = vm.parseUint(proofData.proof.krs.x);
        verifierProof.c.Y = vm.parseUint(proofData.proof.krs.y);

        uint256  [2] memory proofCommitment;
        proofCommitment[0] = vm.parseUint(proofData.proof.commitments[0].x);
        proofCommitment[1] =vm.parseUint(proofData.proof.commitments[0].y);

        bool ret = verifier.verifyTx(verifierProof, input, proofCommitment); 
       
         assert(ret == true); 

    }

    function test_ValidPublicInputs() public {
        string memory root = vm.projectRoot();
        string memory path1 = string.concat(root, "/verifier/snark_proof_with_public_inputs.json");
        string memory json1 = vm.readFile(path1);
        bytes memory publicWitness = json1.parseRaw(".PublicWitness");
        string[] memory pubwit = abi.decode(publicWitness, ( string[]));
        uint256  [2] memory input;
         for (uint256 i = 0; i < pubwit.length; i++ ){
            input[i]  =  vm.parseUint(pubwit[i]); //--> uint256
        }

        string memory path = string.concat(root, "/verifier/public_inputs.json");
        string memory json = vm.readFile(path);

        bytes memory rootBefore = json.parseRaw(".roots_before.root");
        uint32[] memory rootBe = abi.decode(rootBefore, ( uint32[]));
        uint32[8] memory rootb;
        for (uint256 i = 0; i < rootBe.length; i++ ){
             //console.log("--before[i=%d], value:%s", i, rootBe[i]);
             rootb[i] = rootBe[i];
        }
        
        bytes memory rootAfter = json.parseRaw(".roots_after.root");
        uint32[] memory rootAf = abi.decode(rootAfter, ( uint32[]));
        uint32[8] memory roota;
        for (uint256 i = 0; i < rootAf.length; i++ ){
             roota[i] = rootAf[i];
        }
        
        bytes memory userdata = json.parseRaw(".userdata");
        uint8[] memory dataU = abi.decode(userdata, ( uint8[]));
        uint8[40] memory data;
        for (uint256 i = 0; i < dataU.length; i++ ){
             data[i] = dataU[i];
             console.log("--user data[i=%d], value:%s", i, dataU[i]);
        }
       
        uint256 returnNum = verifier.verifyUserData(userdata, rootb, roota);

        assert(returnNum == input[0]); 

    }

}
