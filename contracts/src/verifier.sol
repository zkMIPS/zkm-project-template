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
        vk.alpha = Pairing.G1Point(uint256(17893901124021210272638516546159505340327529763852882834791797783715120736836), uint256(13231467695658339900650768414253718422865673742208133343863421143503462113168));
        vk.beta = Pairing.G2Point([uint256(17887881584875332850937101588175122783613013000081586029290713558009208876939), uint256(5173405259660942311812575171887960300813259764306641231158685689101871053226)], [uint256(15421617451342223732374977604635144339106949166377272589278289042372210038178), uint256(4017392132424093353152388503575757440680696483308117139782333575602591537707)]);
        vk.gamma = Pairing.G2Point([uint256(1290588609385786732164658248069445973512624381080875973387694511433780759175), uint256(12976305822780276357100851134930711058729218483183625693066553997404593228345)], [uint256(18550887073089422994810671436018260406270744572816708595449262508502728046582), uint256(9330344219985993709191598687338119285767947902106034962837030239408815004050)]);
        vk.delta = Pairing.G2Point([uint256(2993044589807208422766651638941259454777038307900585129916365748825379882932), uint256(13132866685093967953405556380011883248024322097351146455602465105530729440852)], [uint256(9287962342478396674958016817894657236291812832698868341924590133380732632404), uint256(18863937240777198026935865424217363800063264366248119243760988932201425732600)]);
        vk.gamma_abc = new Pairing.G1Point[](3);
        vk.gamma_abc[0] = Pairing.G1Point(uint256(19701327339338910895804292089025911598468663156478648178422701350645841255447), uint256(13110634886897473934651042825240895861319233163917095215468572038313498764478));
        vk.gamma_abc[1] = Pairing.G1Point(uint256(1494470184684746103182008386385284997483007826087210853641973278095564194395), uint256(15287893920487755265590292730618597525230562373919215600034902709720476794414));
        vk.gamma_abc[2] = Pairing.G1Point(uint256(21118373401230054986312559037776481326385579291282972713217534432519345349176), uint256(5928106153795853981148997491594430501835099096792005950029918271482575875827));

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

    function verifyUserData(
        uint8[32] memory _userData,
        uint32[8] memory _memRootBefore,
        uint32[8] memory _memRootAfter
    ) public pure returns (uint256) {
        uint256 userData = 0;
        for (uint256 i = 0; i < 32; i++) {
            userData |= uint256(_userData[i]) << (8 * (31 - i));
        }

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