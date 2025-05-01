// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

import {Test} from "forge-std/Test.sol";
import {ZKMVerifier} from "../src/v1.0.0/ZKMVerifierGroth16.sol";

contract ZKMVerifierGroth16Test is Test {
    bytes32 internal constant PROGRAM_VKEY = bytes32(0x00143c96c1238489d5da13def86e11ab0a1b7148b5d63b47983cc4623d05a7c3);
    bytes internal constant PUBLIC_VALUES =
        hex"00000000000000000000000000000000000000000000000000000000000000140000000000000000000000000000000000000000000000000000000000001a6d0000000000000000000000000000000000000000000000000000000000002ac2";
    bytes internal constant PROOF_VALID =
        hex"c7bd17e81b42b0920f93f5096572031b00c58e36aad6191737b8e875f3946f694dd971280c889bbf4e5c3877a454f614b8287310b9215c2c9cc11dcd740d6a8969cee51108c6b9c4e460122fe7979d9751033a69aedee2c58a297197760153e0ef7501201a720394bd672b126804324b5d08c1fe524725ff8b46d3c54cf5d1a231101eaf2d45624093e733cc763541068e6daa5776d6988a336fedb769e00724b4bc8db4232bd538441a1492d23c11e0cc9330273029ebd04a5422187477b5427d0e62b21d3cfdbdc481f8bd6c4ba5f03e483139dedfe3bdf9f452a2b1ae3994d3fef15a123c930f04b43ec79b7199ece0826bb265e8cfbcc447260d03de9cea9fc23c2e";
    bytes internal constant PROOF_INVALID = hex"1b5a112d1e86fe060a33eb57cd5925bd7dc008d32908cdc747fa33650a996d292d4e";

    address internal verifier;

    function setUp() public virtual {
        verifier = address(new ZKMVerifier());
    }

    /// @notice Should succeed when the proof is valid.
    function test_VerifyProof_WhenGroth16() public view {
        ZKMVerifier(verifier).verifyProof(PROGRAM_VKEY, PUBLIC_VALUES, PROOF_VALID);
    }

    /// @notice Should revert when the proof is invalid.
    function test_RevertVerifyProof_WhenGroth16() public view {
        ZKMVerifier(verifier).verifyProof(PROGRAM_VKEY, PUBLIC_VALUES, PROOF_VALID);
    }
}
