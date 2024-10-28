// This file is MIT Licensed.
//
// Copyright 2017 Christian Reitwiessner
// Permission is hereby granted, free of charge, to any person obtaining a copy of this software and associated documentation files (the "Software"), to deal in the Software without restriction, including without limitation the rights to use, copy, modify, merge, publish, distribute, sublicense, and/or sell copies of the Software, and to permit persons to whom the Software is furnished to do so, subject to the following conditions:
// The above copyright notice and this permission notice shall be included in all copies or substantial portions of the Software.
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.
pragma solidity ^0.8.0;
library Pairing {
    struct G1Point {
        uint X;
        uint Y;
    }
    // Encoding of field elements is: X[0] * z + X[1]
    struct G2Point {
        uint[2] X;
        uint[2] Y;
    }
    /// @return the generator of G1
    function P1() pure internal returns (G1Point memory) {
        return G1Point(1, 2);
    }
    /// @return the generator of G2
    function P2() pure internal returns (G2Point memory) {
        return G2Point(
            [10857046999023057135944570762232829481370756359578518086990519993285655852781,
             11559732032986387107991004021392285783925812861821192530917403151452391805634],
            [8495653923123431417604973247489272438418190587263600148770280649306958101930,
             4082367875863433681332203403145435568316851327593401208105741076214120093531]
        );
    }
    /// @return the negation of p, i.e. p.addition(p.negate()) should be zero.
    function negate(G1Point memory p) pure internal returns (G1Point memory) {
        // The prime q in the base field F_q for G1
        uint q = 21888242871839275222246405745257275088696311157297823662689037894645226208583;
        if (p.X == 0 && p.Y == 0)
            return G1Point(0, 0);
        return G1Point(p.X, q - (p.Y % q));
    }
    /// @return r the sum of two points of G1
    function addition(G1Point memory p1, G1Point memory p2) internal view returns (G1Point memory r) {
        uint[4] memory input;
        input[0] = p1.X;
        input[1] = p1.Y;
        input[2] = p2.X;
        input[3] = p2.Y;
        bool success;
        assembly {
            success := staticcall(sub(gas(), 2000), 6, input, 0xc0, r, 0x60)
            // Use "invalid" to make gas estimation work
            switch success case 0 { invalid() }
        }
        require(success);
    }


    /// @return r the product of a point on G1 and a scalar, i.e.
    /// p == p.scalar_mul(1) and p.addition(p) == p.scalar_mul(2) for all points p.
    function scalar_mul(G1Point memory p, uint s) internal view returns (G1Point memory r) {
        uint[3] memory input;
        input[0] = p.X;
        input[1] = p.Y;
        input[2] = s;
        bool success;

        assembly {
            success := staticcall(sub(gas(), 2000), 7, input, 0x80, r, 0x60)
            // Use "invalid" to make gas estimation work
            switch success case 0 { invalid() }
        }

        require (success);
    }
    /// @return the result of computing the pairing check
    /// e(p1[0], p2[0]) *  .... * e(p1[n], p2[n]) == 1
    /// For example pairing([P1(), P1().negate()], [P2(), P2()]) should
    /// return true.
    function pairing(G1Point[] memory p1, G2Point[] memory p2) internal view returns (bool) {
        require(p1.length == p2.length);
        uint elements = p1.length;
        uint inputSize = elements * 6;
        uint[] memory input = new uint[](inputSize);
        for (uint i = 0; i < elements; i++)
        {
            input[i * 6 + 0] = p1[i].X;
            input[i * 6 + 1] = p1[i].Y;
            input[i * 6 + 2] = p2[i].X[1];
            input[i * 6 + 3] = p2[i].X[0];
            input[i * 6 + 4] = p2[i].Y[1];
            input[i * 6 + 5] = p2[i].Y[0];
        }
        uint[1] memory out;
        bool success;

        assembly {
            success := staticcall(sub(gas(), 2000), 8, add(input, 0x20), mul(inputSize, 0x20), out, 0x20)
            // Use "invalid" to make gas estimation work
            // switch success case 0 { invalid() }
        }

        require(success,"no");
        return out[0] != 0;
    }
    /// Convenience method for a pairing check for two pairs.
    function pairingProd2(G1Point memory a1, G2Point memory a2, G1Point memory b1, G2Point memory b2) internal view returns (bool) {
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
            G1Point memory a1, G2Point memory a2,
            G1Point memory b1, G2Point memory b2,
            G1Point memory c1, G2Point memory c2
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
            G1Point memory a1, G2Point memory a2,
            G1Point memory b1, G2Point memory b2,
            G1Point memory c1, G2Point memory c2,
            G1Point memory d1, G2Point memory d2
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
    uint256 constant MASK = ~(uint256(0x7) << 253);
    uint256 constant EMPTY_HASH = 0x3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855;
    uint256 constant CIRCUIT_DIGEST = 17524917735085582509473322861927874526127848298611193781519203348111540784568;

    event VerifyEvent(address user);
    event Value(uint x, uint y);

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
    function verifyingKey() pure internal returns (VerifyingKey memory vk) {
        vk.alpha = Pairing.G1Point(uint256(3822932099788692543893615615066972300406373947723030294258252002713585496403), uint256(8173193854899062432314554250991981779941095778853153816262363990613415549438));
        vk.beta = Pairing.G2Point([uint256(638097608883083313836943541679313429707282911426954640699276722576387081317), uint256(3751723829816740730360448481115251256231372311662960756727455782541092751897)], [uint256(11410702092984585592593659204564657716505484111280606030738425543269372552414), uint256(6736124019075283073386797712805716195791904848505873580520536986689099483753)]);
        vk.gamma = Pairing.G2Point([uint256(5630985036407536091973365882144041701080856810089086912097199130278505300476), uint256(141269356882722294339606379867452136343405487604647939844944847450816904784)], [uint256(18809802681208979768328764137493742888362092447107132123188937200472443571395), uint256(19449249420300963896022532575435844361654327173103650152659708471111966573222)]);
        vk.delta = Pairing.G2Point([uint256(10743744079075485890832820142653878510090993325927655607276366849545485999117), uint256(20563041352001208858608214828504098903816500023729058205172300285213774279156)], [uint256(21710866298309228313317221133542251571423139001776712080498468905281372930623), uint256(19858325399274673578321878556365008726898844025397576551348176974037027887616)]);
        vk.gamma_abc = new Pairing.G1Point[](3);
        vk.gamma_abc[0] = Pairing.G1Point(uint256(2993240122707847476044610966790373125918126867295952817555283201864988081296), uint256(9392632542050338609286308498693880942976264842132074203992101574948415893432));
        vk.gamma_abc[1] = Pairing.G1Point(uint256(20299414895746854112827167089405448748114636668365933695934345955174767233644), uint256(4190649257444976759979904796132450365543389494669187575482379914398880386318));
        vk.gamma_abc[2] = Pairing.G1Point(uint256(19603685552297693441087963249855316257621121917070011387955799042838432008674), uint256(17931964434033440369121316382740485127350295117622696658665726645932862034275));

    }
    function verify(uint[2] memory input, Proof memory proof, uint[2] memory proof_commitment) public view returns (uint) {
        uint256 snark_scalar_field = 21888242871839275222246405745257275088548364400416034343698204186575808495617;

        VerifyingKey memory vk = verifyingKey();
        require(input.length + 1 == vk.gamma_abc.length);
        // Compute the linear combination vk_x
        Pairing.G1Point memory vk_x = Pairing.G1Point(0, 0);
        for (uint i = 0; i < input.length; i++) {
            require(input[i] < snark_scalar_field);
            vk_x = Pairing.addition(vk_x, Pairing.scalar_mul(vk.gamma_abc[i + 1], input[i]));
        }
        Pairing.G1Point memory p_c = Pairing.G1Point(proof_commitment[0], proof_commitment[1]);

        vk_x = Pairing.addition(vk_x, vk.gamma_abc[0]);
        vk_x = Pairing.addition(vk_x, p_c);

        if(!Pairing.pairingProd4(
            proof.a, proof.b,
            Pairing.negate(vk_x), vk.gamma,
            Pairing.negate(proof.c), vk.delta,
            Pairing.negate(vk.alpha), vk.beta)) {
            return 1;
        }

        return 0;
    }
    function verifyTx(
            Proof memory proof, uint[2] memory input
        ,uint[2] memory proof_commitment) public returns (bool r) {

        if (verify(input, proof , proof_commitment) == 0) {
            emit VerifyEvent(msg.sender);
            return true;
        } else {
            return false;
        }

    }

    function calculatePublicInput(
        bytes memory _userData,
        uint32[8] memory _memRootBefore,
        uint32[8] memory _memRootAfter
    ) public pure returns (uint256) {
        bytes32 userData = sha256(_userData);

        uint256 memRootBefore = 0;
        for (uint256 i = 0; i < 8; i++) {
            memRootBefore |= uint256(_memRootBefore[i]) << (32 * (7 - i));
        }
        uint256 memRootAfter = 0;
        for (uint256 i = 0; i < 8; i++) {
            memRootAfter |= uint256(_memRootAfter[i]) << (32 * (7 - i));
        }

        bytes memory dataToHash = abi.encodePacked(
            memRootBefore,
            memRootAfter,
            userData,
            CIRCUIT_DIGEST,
            getConstantSigmasCap()
        );

        uint256 hash_o = uint256(sha256(dataToHash)) & MASK;
        uint256 hashValue = uint256(sha256(abi.encodePacked(EMPTY_HASH,hash_o))) & MASK;

        return hashValue;
    }

    function getConstantSigmasCap() public pure returns (uint256[16] memory) {
        return [
                        91383584712019272681377229182172954852761112840230485576576219640999915415908,
                        59255968443781732618398964440087269720283395714373383774650673994811543257891,
                        43522785206513529710314565972458653794534948943367780523535399673207256542667,
                        40342750684549940679043033475171365072773076053340772573915667403010979823501,
                        27317612392857105781687183282424868606341901129358521256675721204392832410439,
                        43307034001377032728710254791751651528006774034804994236242404941502133194788,
                        96847759610594182728575775013319360940143540443603110434763689062442262276056,
                        103816866397357498456560831382884881537689393646419658130688223819092141266482,
                        92927370392150370847456421756552282870716539874859278967093815334673659047245,
                        48196270063118112994752430351531600491056188761932189573202142214229711823214,
                        72300508929423173342535162393269108469073018007670992928238766269813213409236,
                        96435373004420944780577491829986863106913456501189307806899876787207854460461,
                        49751597273171012664300923220213149590083424576410050544981754459509775460226,
                        34852111753877916137017959036830401094053286275843684490553560491310958812859,
                        50803581021794237060724898598173476602507622641970514917690126680609140114800,
                        97509144034546411327314284027738859064756953812076402994654732676369028259664
                ];
    }
}