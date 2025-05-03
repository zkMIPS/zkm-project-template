// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.13;

import {Script, console} from "forge-std/Script.sol";
import {ZKMVerifier} from "../src/v1.0.0/ZKMVerifierPlonk.sol";

contract ZKMVerifierPlonkScript is Script {
    ZKMVerifier public verifier;

    function setUp() public {}

    function run() public {
        vm.startBroadcast();

        verifier = new ZKMVerifier();

        vm.stopBroadcast();
    }
}
