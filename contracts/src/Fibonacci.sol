// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

import {IZKMVerifier} from "./IZKMVerifier.sol";

struct PublicValuesStruct {
    uint32 n;
    uint32 a;
    uint32 b;
}

/// @title Fibonacci.
/// @author ZKM
/// @notice This contract implements a simple example of verifying the proof of a computing a
///         fibonacci number.
contract Fibonacci {
    /// @notice The address of the zkMIPS verifier contract.
    /// @dev This is a specific ZKMVerifier for a specific version.
    IZKMVerifier public verifier;

    /// @notice The verification key for the fibonacci program.
    bytes32 public fibonacciProgramVKey;

    constructor(IZKMVerifier _verifier, bytes32 _fibonacciProgramVKey) {
        verifier = _verifier;
        fibonacciProgramVKey = _fibonacciProgramVKey;
    }

    /// @notice The entrypoint for verifying the proof of a fibonacci number.
    /// @param _proofBytes The encoded proof.
    /// @param _publicValues The encoded public values.
    function verifyFibonacciProof(bytes calldata _publicValues, bytes calldata _proofBytes)
        public
        view
        returns (uint32, uint32, uint32)
    {
        IZKMVerifier(verifier).verifyProof(fibonacciProgramVKey, _publicValues, _proofBytes);
        PublicValuesStruct memory publicValues = abi.decode(_publicValues, (PublicValuesStruct));
        return (publicValues.n, publicValues.a, publicValues.b);
    }
}
