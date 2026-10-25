// SPDX-License-Identifier: LicenseRef-DCL-1.0
// SPDX-FileCopyrightText: Copyright (c) 2020 Rain Open Source Software Ltd
pragma solidity =0.8.25;

import {RaindexV6ExternalRealTest} from "test/util/abstract/RaindexV6ExternalRealTest.sol";
import {
    OrderConfigV4,
    OrderV4,
    TaskV2,
    ClearConfigV2,
    SignedContextV1,
    IOV2,
    EvaluableV4
} from "raindex-interface-0.1.10/src/interface/IRaindexV7.sol";
import {IRaindexV6} from "raindex-interface-0.1.10/src/interface/deprecated/v6/IRaindexV6.sol";
import {IERC20} from "@openzeppelin-contracts-5.7.0/token/ERC20/IERC20.sol";
import {MessageHashUtils} from "@openzeppelin-contracts-5.7.0/utils/cryptography/MessageHashUtils.sol";
import {Strings} from "@openzeppelin-contracts-5.7.0/utils/Strings.sol";
import {LibDecimalFloat, Float} from "rain-math-float-0.3.10/src/lib/LibDecimalFloat.sol";

/// @title RaindexV6ClearSignedContextRoutingTest
/// @notice `clear4` evaluates alice's order with `aliceSignedContext` and bob's
/// with `bobSignedContext`, matching `quote2`/`takeOrders4`. Each order's
/// expression asserts the signer and signed content it sees, so routing is
/// pinned by signer and content rather than slot. Runs against fresh-compiled
/// RaindexV6 so source mutations are observable.
contract RaindexV6ClearSignedContextRoutingTest is RaindexV6ExternalRealTest {
    using Strings for address;
    using Strings for uint256;
    using LibDecimalFloat for Float;

    bytes32 internal constant ALICE_IN = bytes32(uint256(0xa1));
    bytes32 internal constant ALICE_OUT = bytes32(uint256(0xa2));
    bytes32 internal constant BOB_IN = bytes32(uint256(0xb1));
    bytes32 internal constant BOB_OUT = bytes32(uint256(0xb2));

    function setUp() external {
        vm.etch(address(iRaindex), vm.getDeployedCode("RaindexV6.sol:RaindexV6"));
    }

    function signedContext(uint256 pk, bytes32 content) internal pure returns (SignedContextV1[] memory) {
        bytes32[] memory context = new bytes32[](1);
        context[0] = content;
        (uint8 v, bytes32 r, bytes32 s) =
            vm.sign(pk, MessageHashUtils.toEthSignedMessageHash(keccak256(abi.encodePacked(context))));
        SignedContextV1[] memory signedContexts = new SignedContextV1[](1);
        signedContexts[0] =
            SignedContextV1({signer: vm.addr(pk), context: context, signature: abi.encodePacked(r, s, v)});
        return signedContexts;
    }

    function expectingBytecode(string memory name, address expectedSigner, bytes32 expectedContent)
        internal
        view
        returns (bytes memory)
    {
        return iParserV2.parse2(
            bytes(
                string.concat(
                    "using-words-from ",
                    address(iSubParser).toHexString(),
                    "\n:ensure(equal-to(signer<0>() ",
                    expectedSigner.toHexString(),
                    ") \"",
                    name,
                    " signer\"),\n:ensure(equal-to(signed-context<0 0>() ",
                    uint256(expectedContent).toHexString(),
                    ") \"",
                    name,
                    " signed context\"),\n_ _: 1 1;\n:;"
                )
            )
        );
    }

    function addFundedOrder(
        address owner,
        address inputToken,
        bytes32 inputVault,
        address outputToken,
        bytes32 outputVault,
        bytes memory bytecode
    ) internal returns (OrderV4 memory) {
        IOV2[] memory inputs = new IOV2[](1);
        inputs[0] = IOV2({token: inputToken, vaultId: inputVault});
        IOV2[] memory outputs = new IOV2[](1);
        outputs[0] = IOV2({token: outputToken, vaultId: outputVault});
        OrderConfigV4 memory config = OrderConfigV4({
            evaluable: EvaluableV4({bytecode: bytecode, interpreter: iInterpreter, store: iStore}),
            validInputs: inputs,
            validOutputs: outputs,
            nonce: 0,
            secret: 0,
            meta: ""
        });

        vm.mockCall(
            outputToken,
            abi.encodeWithSelector(IERC20.transferFrom.selector, owner, address(iRaindex)),
            abi.encode(true)
        );
        vm.startPrank(owner);
        iRaindex.deposit4(outputToken, outputVault, LibDecimalFloat.packLossless(100, 0), new TaskV2[](0));
        iRaindex.addOrder4(config, new TaskV2[](0));
        vm.stopPrank();

        return OrderV4(owner, config.evaluable, config.validInputs, config.validOutputs, config.nonce);
    }

    function aliceOrderExpecting(uint256 signerPk, bytes32 content) internal returns (OrderV4 memory) {
        bytes memory bytecode = expectingBytecode("alice", vm.addr(signerPk), content);
        return addFundedOrder(makeAddr("alice"), address(iToken0), ALICE_IN, address(iToken1), ALICE_OUT, bytecode);
    }

    function bobOrderExpecting(uint256 signerPk, bytes32 content) internal returns (OrderV4 memory) {
        bytes memory bytecode = expectingBytecode("bob", vm.addr(signerPk), content);
        return addFundedOrder(makeAddr("bob"), address(iToken1), BOB_IN, address(iToken0), BOB_OUT, bytecode);
    }

    function orders(uint256 aliceSignerPk, bytes32 aliceContent, uint256 bobSignerPk, bytes32 bobContent)
        internal
        returns (OrderV4 memory, OrderV4 memory)
    {
        return (aliceOrderExpecting(aliceSignerPk, aliceContent), bobOrderExpecting(bobSignerPk, bobContent));
    }

    function boundPk(uint256 pk) internal pure returns (uint256) {
        return bound(pk, 1, SECP256K1_ORDER - 1);
    }

    /// forge-config: default.fuzz.runs = 32
    function testClearRoutesEachSignedContextToItsOwnOrder(
        uint256 aliceSignerPk,
        bytes32 aliceContent,
        uint256 bobSignerPk,
        bytes32 bobContent
    ) external {
        aliceSignerPk = boundPk(aliceSignerPk);
        bobSignerPk = boundPk(bobSignerPk);
        vm.assume(vm.addr(aliceSignerPk) != vm.addr(bobSignerPk));
        (OrderV4 memory aliceOrder, OrderV4 memory bobOrder) =
            orders(aliceSignerPk, aliceContent, bobSignerPk, bobContent);

        iRaindex.clear4(
            aliceOrder,
            bobOrder,
            ClearConfigV2(0, 0, 0, 0, bytes32(uint256(0xc1)), bytes32(uint256(0xc2))),
            signedContext(aliceSignerPk, aliceContent),
            signedContext(bobSignerPk, bobContent)
        );

        assertTrue(iRaindex.vaultBalance2(aliceOrder.owner, address(iToken0), ALICE_IN).gt(LibDecimalFloat.FLOAT_ZERO));
        assertTrue(iRaindex.vaultBalance2(bobOrder.owner, address(iToken1), BOB_IN).gt(LibDecimalFloat.FLOAT_ZERO));
    }

    /// forge-config: default.fuzz.runs = 32
    function testClearRevertsWhenSignedContextsAreSwapped(
        uint256 aliceSignerPk,
        bytes32 aliceContent,
        uint256 bobSignerPk,
        bytes32 bobContent
    ) external {
        aliceSignerPk = boundPk(aliceSignerPk);
        bobSignerPk = boundPk(bobSignerPk);
        vm.assume(vm.addr(aliceSignerPk) != vm.addr(bobSignerPk));
        (OrderV4 memory aliceOrder, OrderV4 memory bobOrder) =
            orders(aliceSignerPk, aliceContent, bobSignerPk, bobContent);
        ClearConfigV2 memory clearConfig = ClearConfigV2(0, 0, 0, 0, bytes32(uint256(0xc1)), bytes32(uint256(0xc2)));
        SignedContextV1[] memory aliceSignedContext = signedContext(aliceSignerPk, aliceContent);
        SignedContextV1[] memory bobSignedContext = signedContext(bobSignerPk, bobContent);

        vm.expectRevert("alice signer");
        iRaindex.clear4(aliceOrder, bobOrder, clearConfig, bobSignedContext, aliceSignedContext);
    }

    function testClear3SelectorIsRemoved() external {
        (bool success, bytes memory returnData) = address(iRaindex)
            .call(
                abi.encodeCall(
                    IRaindexV6.clear3,
                    (
                        OrderV4(address(0), EvaluableV4(iInterpreter, iStore, ""), new IOV2[](0), new IOV2[](0), 0),
                        OrderV4(address(0), EvaluableV4(iInterpreter, iStore, ""), new IOV2[](0), new IOV2[](0), 0),
                        ClearConfigV2(0, 0, 0, 0, 0, 0),
                        new SignedContextV1[](0),
                        new SignedContextV1[](0)
                    )
                )
            );
        assertFalse(success);
        assertEq(returnData.length, 0);
    }
}
