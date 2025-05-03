// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

import {Test, console} from "forge-std/Test.sol";
import {stdJson} from "forge-std/StdJson.sol";
import {Fibonacci} from "../src/Fibonacci.sol";
import {ZKMVerifier as ZKMVerifierGroth16} from "../src/v1.0.0/ZKMVerifierGroth16.sol";
import {ZKMVerifier as ZKMVerifierPlonk} from "../src/v1.0.0/ZKMVerifierPlonk.sol";

struct ZKMProofFixtureJson {
    uint32 a;
    uint32 b;
    uint32 n;
    bytes proof;
    bytes publicValues;
    bytes32 vkey;
}

contract FibonacciGroth16Test is Test {
    using stdJson for string;

    ZKMVerifierGroth16 public verifier;
    Fibonacci public fibonacci;

    function loadFixture() public view returns (ZKMProofFixtureJson memory) {
        string memory root = vm.projectRoot();
        string memory path = string.concat(root, "/src/fixtures/groth16-fixture.json");
        string memory json = vm.readFile(path);
        bytes memory jsonBytes = json.parseRaw(".");
        return abi.decode(jsonBytes, (ZKMProofFixtureJson));
    }

    function setUp() public {
        ZKMProofFixtureJson memory fixture = loadFixture();

        verifier = new ZKMVerifierGroth16();
        fibonacci = new Fibonacci(verifier, fixture.vkey);
    }

    function test_ValidFibonacciProof() public {
        ZKMProofFixtureJson memory fixture = loadFixture();

        vm.mockCall(
            address(verifier), abi.encodeWithSelector(ZKMVerifierGroth16.verifyProof.selector), abi.encode(true)
        );

        (uint32 n, uint32 a, uint32 b) = fibonacci.verifyFibonacciProof(fixture.publicValues, fixture.proof);
        assert(n == fixture.n);
        assert(a == fixture.a);
        assert(b == fixture.b);
    }

    function testRevert_InvalidFibonacciProof() public {
        vm.expectRevert();

        ZKMProofFixtureJson memory fixture = loadFixture();

        // Create a fake proof.
        bytes memory fakeProof = new bytes(fixture.proof.length);

        fibonacci.verifyFibonacciProof(fixture.publicValues, fakeProof);
    }
}

contract FibonacciPlonkTest is Test {
    using stdJson for string;

    ZKMVerifierPlonk public verifier;
    Fibonacci public fibonacci;

    function loadFixture() public view returns (ZKMProofFixtureJson memory) {
        string memory root = vm.projectRoot();
        string memory path = string.concat(root, "/src/fixtures/plonk-fixture.json");
        string memory json = vm.readFile(path);
        bytes memory jsonBytes = json.parseRaw(".");
        return abi.decode(jsonBytes, (ZKMProofFixtureJson));
    }

    function setUp() public {
        ZKMProofFixtureJson memory fixture = loadFixture();

        verifier = new ZKMVerifierPlonk();
        fibonacci = new Fibonacci(verifier, fixture.vkey);
    }

    function test_ValidFibonacciProof() public {
        ZKMProofFixtureJson memory fixture = loadFixture();

        vm.mockCall(address(verifier), abi.encodeWithSelector(ZKMVerifierPlonk.verifyProof.selector), abi.encode(true));

        (uint32 n, uint32 a, uint32 b) = fibonacci.verifyFibonacciProof(fixture.publicValues, fixture.proof);
        assert(n == fixture.n);
        assert(a == fixture.a);
        assert(b == fixture.b);
    }

    function testRevert_InvalidFibonacciProof() public {
        vm.expectRevert();

        ZKMProofFixtureJson memory fixture = loadFixture();

        // Create a fake proof.
        bytes memory fakeProof = new bytes(fixture.proof.length);

        fibonacci.verifyFibonacciProof(fixture.publicValues, fakeProof);
    }
}
