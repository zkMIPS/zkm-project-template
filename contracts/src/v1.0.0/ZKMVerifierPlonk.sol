// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

import {IZKMVerifier, IZKMVerifierWithHash} from "../IZKMVerifier.sol";
import {PlonkVerifier} from "./PlonkVerifier.sol";

/// @title zkMIPS Verifier
/// @author ZKM
/// @notice This contracts implements a solidity verifier for zkMIPS.
contract ZKMVerifier is PlonkVerifier, IZKMVerifierWithHash {
    /// @notice Thrown when the verifier selector from this proof does not match the one in this
    /// verifier. This indicates that this proof was sent to the wrong verifier.
    /// @param received The verifier selector from the first 4 bytes of the proof.
    /// @param expected The verifier selector from the first 4 bytes of the VERIFIER_HASH().
    error WrongVerifierSelector(bytes4 received, bytes4 expected);

    /// @notice Thrown when the proof is invalid.
    error InvalidProof();

    function VERSION() external pure returns (string memory) {
        return "v1.0.0";
    }

    /// @inheritdoc IZKMVerifierWithHash
    function VERIFIER_HASH() public pure returns (bytes32) {
        return 0x2eb5d34c694419038fe1601c6bd03441e17b1e4bb580a5d3726e57d364176f63;
    }

    /// @notice Hashes the public values to a field elements inside Bn254.
    /// @param publicValues The public values.
    function hashPublicValues(bytes calldata publicValues) public pure returns (bytes32) {
        return sha256(publicValues) & bytes32(uint256((1 << 253) - 1));
    }

    /// @notice Verifies a proof with given public values and vkey.
    /// @param programVKey The verification key for the MIPS program.
    /// @param publicValues The public values encoded as bytes.
    /// @param proofBytes The proof of the program execution the zkMIPS zkVM encoded as bytes.
    function verifyProof(bytes32 programVKey, bytes calldata publicValues, bytes calldata proofBytes) external view {
        bytes4 receivedSelector = bytes4(proofBytes[:4]);
        bytes4 expectedSelector = bytes4(VERIFIER_HASH());
        if (receivedSelector != expectedSelector) {
            revert WrongVerifierSelector(receivedSelector, expectedSelector);
        }

        bytes32 publicValuesDigest = hashPublicValues(publicValues);
        uint256[] memory inputs = new uint256[](2);
        inputs[0] = uint256(programVKey);
        inputs[1] = uint256(publicValuesDigest);
        bool success = this.Verify(proofBytes[4:], inputs);
        if (!success) {
            revert InvalidProof();
        }
    }
}
