// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.13;

import {Script, console} from "forge-std/Script.sol";
import {ZKMVerifier} from "../src/v1.0.0/ZKMVerifierGroth16.sol";

contract ZKMVerifierGroth16Script is Script {
    ZKMVerifier public verifier;

    function setUp() public {}

    function run() public {
        vm.startBroadcast();

        verifier = new ZKMVerifier();

        vm.stopBroadcast();
    }
}
