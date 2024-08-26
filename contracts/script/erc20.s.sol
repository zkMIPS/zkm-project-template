// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.13;

import {Script, console} from "forge-std/Script.sol";
import {ERC20Token} from "../src/erc20.sol";

contract ERC20TokenScript is Script {
    ERC20Token public erc20;

    function setUp() public {}

    function run() public {
        vm.startBroadcast();

        erc20 = new ERC20Token();

        vm.stopBroadcast();
    }
}
