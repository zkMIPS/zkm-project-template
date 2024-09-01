// This file is MIT Licensed.
//
// Copyright 2017 Christian Reitwiessner
// Permission is hereby granted, free of charge, to any person obtaining a copy of this software and associated documentation files (the "Software"), to deal in the Software without restriction, including without limitation the rights to use, copy, modify, merge, publish, distribute, sublicense, and/or sell copies of the Software, and to permit persons to whom the Software is furnished to do so, subject to the following conditions:
// The above copyright notice and this permission notice shall be included in all copies or substantial portions of the Software.
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.
pragma solidity ^0.8.0;

library Pairing {
    struct G1Point {
        uint256 X;
        uint256 Y;
    }
    // Encoding of field elements is: X[0] * z + X[1]

    struct G2Point {
        uint256[2] X;
        uint256[2] Y;
    }
    /// @return the generator of G1

    function P1() internal pure returns (G1Point memory) {
        return G1Point(1, 2);
    }
    /// @return the generator of G2

    function P2() internal pure returns (G2Point memory) {
        return G2Point(
            [
                10857046999023057135944570762232829481370756359578518086990519993285655852781,
                11559732032986387107991004021392285783925812861821192530917403151452391805634
            ],
            [
                8495653923123431417604973247489272438418190587263600148770280649306958101930,
                4082367875863433681332203403145435568316851327593401208105741076214120093531
            ]
        );
    }
    /// @return the negation of p, i.e. p.addition(p.negate()) should be zero.

    function negate(G1Point memory p) internal pure returns (G1Point memory) {
        // The prime q in the base field F_q for G1
        uint256 q = 21888242871839275222246405745257275088696311157297823662689037894645226208583;
        if (p.X == 0 && p.Y == 0) {
            return G1Point(0, 0);
        }
        return G1Point(p.X, q - (p.Y % q));
    }
    /// @return r the sum of two points of G1

    function addition(G1Point memory p1, G1Point memory p2) internal view returns (G1Point memory r) {
        uint256[4] memory input;
        input[0] = p1.X;
        input[1] = p1.Y;
        input[2] = p2.X;
        input[3] = p2.Y;
        bool success;
        assembly {
            success := staticcall(sub(gas(), 2000), 6, input, 0xc0, r, 0x60)
            // Use "invalid" to make gas estimation work
            switch success
            case 0 { invalid() }
        }
        require(success);
    }

    /// @return r the product of a point on G1 and a scalar, i.e.
    /// p == p.scalar_mul(1) and p.addition(p) == p.scalar_mul(2) for all points p.
    function scalar_mul(G1Point memory p, uint256 s) internal view returns (G1Point memory r) {
        uint256[3] memory input;
        input[0] = p.X;
        input[1] = p.Y;
        input[2] = s;
        bool success;

        assembly {
            success := staticcall(sub(gas(), 2000), 7, input, 0x80, r, 0x60)
            // Use "invalid" to make gas estimation work
            switch success
            case 0 { invalid() }
        }

        require(success);
    }
    /// @return the result of computing the pairing check
    /// e(p1[0], p2[0]) *  .... * e(p1[n], p2[n]) == 1
    /// For example pairing([P1(), P1().negate()], [P2(), P2()]) should
    /// return true.

    function pairing(G1Point[] memory p1, G2Point[] memory p2) internal view returns (bool) {
        require(p1.length == p2.length);
        uint256 elements = p1.length;
        uint256 inputSize = elements * 6;
        uint256[] memory input = new uint256[](inputSize);
        for (uint256 i = 0; i < elements; i++) {
            input[i * 6 + 0] = p1[i].X;
            input[i * 6 + 1] = p1[i].Y;
            input[i * 6 + 2] = p2[i].X[1];
            input[i * 6 + 3] = p2[i].X[0];
            input[i * 6 + 4] = p2[i].Y[1];
            input[i * 6 + 5] = p2[i].Y[0];
        }
        uint256[1] memory out;
        bool success;

        assembly {
            success := staticcall(sub(gas(), 2000), 8, add(input, 0x20), mul(inputSize, 0x20), out, 0x20)
            // Use "invalid" to make gas estimation work
            // switch success case 0 { invalid() }
        }

        require(success, "no");
        return out[0] != 0;
    }
    /// Convenience method for a pairing check for two pairs.

    function pairingProd2(G1Point memory a1, G2Point memory a2, G1Point memory b1, G2Point memory b2)
        internal
        view
        returns (bool)
    {
        G1Point[] memory p1 = new G1Point[](2);
        G2Point[] memory p2 = new G2Point[](2);
        p1[0] = a1;
        p1[1] = b1;
        p2[0] = a2;
        p2[1] = b2;
        return pairing(p1, p2);
    }
    /// Convenience method for a pairing check for three pairs.

    function pairingProd3(
        G1Point memory a1,
        G2Point memory a2,
        G1Point memory b1,
        G2Point memory b2,
        G1Point memory c1,
        G2Point memory c2
    ) internal view returns (bool) {
        G1Point[] memory p1 = new G1Point[](3);
        G2Point[] memory p2 = new G2Point[](3);
        p1[0] = a1;
        p1[1] = b1;
        p1[2] = c1;
        p2[0] = a2;
        p2[1] = b2;
        p2[2] = c2;
        return pairing(p1, p2);
    }
    /// Convenience method for a pairing check for four pairs.

    function pairingProd4(
        G1Point memory a1,
        G2Point memory a2,
        G1Point memory b1,
        G2Point memory b2,
        G1Point memory c1,
        G2Point memory c2,
        G1Point memory d1,
        G2Point memory d2
    ) internal view returns (bool) {
        G1Point[] memory p1 = new G1Point[](4);
        G2Point[] memory p2 = new G2Point[](4);
        p1[0] = a1;
        p1[1] = b1;
        p1[2] = c1;
        p1[3] = d1;
        p2[0] = a2;
        p2[1] = b2;
        p2[2] = c2;
        p2[3] = d2;
        return pairing(p1, p2);
    }
}

contract Verifier {
    event VerifyEvent(address user);
    event Value(uint256 x, uint256 y);

    using Pairing for *;

    struct VerifyingKey {
        Pairing.G1Point alpha;
        Pairing.G2Point beta;
        Pairing.G2Point gamma;
        Pairing.G2Point delta;
        Pairing.G1Point[] gamma_abc;
    }

    struct Proof {
        Pairing.G1Point a;
        Pairing.G2Point b;
        Pairing.G1Point c;
    }

    function verifyingKey() internal pure returns (VerifyingKey memory vk) {
        vk.alpha = Pairing.G1Point(
            uint256(1302117677361910542468231459920151979091943974637170989682078414965483188281),
            uint256(3867027232541254772075321782990110912188042242600509887048224814657257159698)
        );
        vk.beta = Pairing.G2Point(
            [
                uint256(14801598391870170737198692701386770912604845156083007898115207806880331104566),
                uint256(8887197963407115464952338401821009369268043467003949766013809662131496593072)
            ],
            [
                uint256(2302950197816344635982649983475492746659754717141582763685467658478763816306),
                uint256(14546187023230700546432589234473482335485386237848823348478034929543161739283)
            ]
        );
        vk.gamma = Pairing.G2Point(
            [
                uint256(21193141255951568057241429222082940283695501709413235369564469208569004052793),
                uint256(3543300415735709873986269632293216615803290229738110628079419132119895168426)
            ],
            [
                uint256(19424433138581986634666612545039047288274363442950180138503536129758457024677),
                uint256(6362422927587009743413699214616594497342377797306122883504232193179957350849)
            ]
        );
        vk.delta = Pairing.G2Point(
            [
                uint256(7385068669210247546745717886468250961916074106623897850964828221823027078300),
                uint256(19192861161778484819667846292071148227610103974433548788489138394128409122900)
            ],
            [
                uint256(16629788660241964719751663969097289423751591856184503635027243409639557586033),
                uint256(12621057362679150933080455091106081399032880392542297635602848266829657178901)
            ]
        );
        vk.gamma_abc = new Pairing.G1Point[](66);
        vk.gamma_abc[0] = Pairing.G1Point(
            uint256(8227080285934790069042597020674361445558585705178995755056818730155727831239),
            uint256(10964499248881051681906280027475314624544858740993358611805036213651538481961)
        );
        vk.gamma_abc[1] = Pairing.G1Point(uint256(0), uint256(0));
        vk.gamma_abc[2] = Pairing.G1Point(uint256(0), uint256(0));
        vk.gamma_abc[3] = Pairing.G1Point(uint256(0), uint256(0));
        vk.gamma_abc[4] = Pairing.G1Point(uint256(0), uint256(0));
        vk.gamma_abc[5] = Pairing.G1Point(uint256(0), uint256(0));
        vk.gamma_abc[6] = Pairing.G1Point(uint256(0), uint256(0));
        vk.gamma_abc[7] = Pairing.G1Point(uint256(0), uint256(0));
        vk.gamma_abc[8] = Pairing.G1Point(uint256(0), uint256(0));
        vk.gamma_abc[9] = Pairing.G1Point(uint256(0), uint256(0));
        vk.gamma_abc[10] = Pairing.G1Point(uint256(0), uint256(0));
        vk.gamma_abc[11] = Pairing.G1Point(uint256(0), uint256(0));
        vk.gamma_abc[12] = Pairing.G1Point(uint256(0), uint256(0));
        vk.gamma_abc[13] = Pairing.G1Point(uint256(0), uint256(0));
        vk.gamma_abc[14] = Pairing.G1Point(uint256(0), uint256(0));
        vk.gamma_abc[15] = Pairing.G1Point(uint256(0), uint256(0));
        vk.gamma_abc[16] = Pairing.G1Point(uint256(0), uint256(0));
        vk.gamma_abc[17] = Pairing.G1Point(uint256(0), uint256(0));
        vk.gamma_abc[18] = Pairing.G1Point(uint256(0), uint256(0));
        vk.gamma_abc[19] = Pairing.G1Point(uint256(0), uint256(0));
        vk.gamma_abc[20] = Pairing.G1Point(uint256(0), uint256(0));
        vk.gamma_abc[21] = Pairing.G1Point(uint256(0), uint256(0));
        vk.gamma_abc[22] = Pairing.G1Point(uint256(0), uint256(0));
        vk.gamma_abc[23] = Pairing.G1Point(uint256(0), uint256(0));
        vk.gamma_abc[24] = Pairing.G1Point(uint256(0), uint256(0));
        vk.gamma_abc[25] = Pairing.G1Point(uint256(0), uint256(0));
        vk.gamma_abc[26] = Pairing.G1Point(uint256(0), uint256(0));
        vk.gamma_abc[27] = Pairing.G1Point(uint256(0), uint256(0));
        vk.gamma_abc[28] = Pairing.G1Point(uint256(0), uint256(0));
        vk.gamma_abc[29] = Pairing.G1Point(uint256(0), uint256(0));
        vk.gamma_abc[30] = Pairing.G1Point(uint256(0), uint256(0));
        vk.gamma_abc[31] = Pairing.G1Point(uint256(0), uint256(0));
        vk.gamma_abc[32] = Pairing.G1Point(uint256(0), uint256(0));
        vk.gamma_abc[33] = Pairing.G1Point(uint256(0), uint256(0));
        vk.gamma_abc[34] = Pairing.G1Point(uint256(0), uint256(0));
        vk.gamma_abc[35] = Pairing.G1Point(uint256(0), uint256(0));
        vk.gamma_abc[36] = Pairing.G1Point(uint256(0), uint256(0));
        vk.gamma_abc[37] = Pairing.G1Point(uint256(0), uint256(0));
        vk.gamma_abc[38] = Pairing.G1Point(uint256(0), uint256(0));
        vk.gamma_abc[39] = Pairing.G1Point(uint256(0), uint256(0));
        vk.gamma_abc[40] = Pairing.G1Point(uint256(0), uint256(0));
        vk.gamma_abc[41] = Pairing.G1Point(uint256(0), uint256(0));
        vk.gamma_abc[42] = Pairing.G1Point(uint256(0), uint256(0));
        vk.gamma_abc[43] = Pairing.G1Point(uint256(0), uint256(0));
        vk.gamma_abc[44] = Pairing.G1Point(uint256(0), uint256(0));
        vk.gamma_abc[45] = Pairing.G1Point(uint256(0), uint256(0));
        vk.gamma_abc[46] = Pairing.G1Point(uint256(0), uint256(0));
        vk.gamma_abc[47] = Pairing.G1Point(uint256(0), uint256(0));
        vk.gamma_abc[48] = Pairing.G1Point(uint256(0), uint256(0));
        vk.gamma_abc[49] = Pairing.G1Point(uint256(0), uint256(0));
        vk.gamma_abc[50] = Pairing.G1Point(uint256(0), uint256(0));
        vk.gamma_abc[51] = Pairing.G1Point(uint256(0), uint256(0));
        vk.gamma_abc[52] = Pairing.G1Point(uint256(0), uint256(0));
        vk.gamma_abc[53] = Pairing.G1Point(uint256(0), uint256(0));
        vk.gamma_abc[54] = Pairing.G1Point(uint256(0), uint256(0));
        vk.gamma_abc[55] = Pairing.G1Point(uint256(0), uint256(0));
        vk.gamma_abc[56] = Pairing.G1Point(uint256(0), uint256(0));
        vk.gamma_abc[57] = Pairing.G1Point(uint256(0), uint256(0));
        vk.gamma_abc[58] = Pairing.G1Point(uint256(0), uint256(0));
        vk.gamma_abc[59] = Pairing.G1Point(uint256(0), uint256(0));
        vk.gamma_abc[60] = Pairing.G1Point(uint256(0), uint256(0));
        vk.gamma_abc[61] = Pairing.G1Point(uint256(0), uint256(0));
        vk.gamma_abc[62] = Pairing.G1Point(uint256(0), uint256(0));
        vk.gamma_abc[63] = Pairing.G1Point(uint256(0), uint256(0));
        vk.gamma_abc[64] = Pairing.G1Point(uint256(0), uint256(0));
        vk.gamma_abc[65] = Pairing.G1Point(
            uint256(4811611970921445049598943815770920513454731152861450007955704045904060821183),
            uint256(5288069492546863186831394199978690427589784869292363777070175867300583082895)
        );
    }

    function verify(uint256[65] memory input, Proof memory proof, uint256[2] memory proof_commitment)
        public
        view
        returns (uint256)
    {
        uint256 snark_scalar_field = 21888242871839275222246405745257275088548364400416034343698204186575808495617;

        VerifyingKey memory vk = verifyingKey();
        require(input.length + 1 == vk.gamma_abc.length);
        // Compute the linear combination vk_x
        Pairing.G1Point memory vk_x = Pairing.G1Point(0, 0);
        for (uint256 i = 0; i < input.length; i++) {
            require(input[i] < snark_scalar_field);
            vk_x = Pairing.addition(vk_x, Pairing.scalar_mul(vk.gamma_abc[i + 1], input[i]));
        }
        Pairing.G1Point memory p_c = Pairing.G1Point(proof_commitment[0], proof_commitment[1]);

        vk_x = Pairing.addition(vk_x, vk.gamma_abc[0]);
        vk_x = Pairing.addition(vk_x, p_c);

        if (
            !Pairing.pairingProd4(
                proof.a,
                proof.b,
                Pairing.negate(vk_x),
                vk.gamma,
                Pairing.negate(proof.c),
                vk.delta,
                Pairing.negate(vk.alpha),
                vk.beta
            )
        ) {
            return 1;
        }

        return 0;
    }

    function verifyTx(Proof memory proof, uint256[65] memory input, uint256[2] memory proof_commitment)
        public
        returns (bool r)
    {
        if (verify(input, proof, proof_commitment) == 0) {
            emit VerifyEvent(msg.sender);
            return true;
        } else {
            return false;
        }
    }
}
