// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.13;

import {Test, console} from "forge-std/Test.sol";
import {ERC20Token} from "../src/erc20.sol";

contract ERC20TokenTest is Test {
    ERC20Token public erc20;

    function setUp() public {
        erc20 = new ERC20Token();
    }

    function test_mintWithProof() public { //TBD
    }
}
