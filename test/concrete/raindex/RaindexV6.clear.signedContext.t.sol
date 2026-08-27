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
} from "raindex-interface-0.1.3/src/interface/IRaindexV6.sol";
import {IERC20} from "@openzeppelin-contracts-5.6.1/token/ERC20/IERC20.sol";
import {Strings} from "@openzeppelin-contracts-5.6.1/utils/Strings.sol";
import {MessageHashUtils} from "@openzeppelin-contracts-5.6.1/utils/cryptography/MessageHashUtils.sol";
import {LibHashNoAlloc} from "rain-lib-hash-0.1.0/src/LibHashNoAlloc.sol";
import {LibDecimalFloat} from "rain-math-float-0.1.1/src/lib/LibDecimalFloat.sol";
import {stdError} from "forge-std-1.16.2/src/StdError.sol";

/// Pins `clear3`'s signed context routing as shipped: alice's evals receive
/// `bobSignedContext` and bob's evals receive `aliceSignedContext` — each
/// order's evals see the signed context supplied for the COUNTERPARTY, the
/// opposite of the `IRaindexV6` NatSpec and of how `quote`/`takeOrders` route
/// signed context to the order being evaluated.
/// https://github.com/rainlanguage/raindex/issues/2850 rules that V6 keeps
/// this routing and a future interface version fixes it under a new
/// entrypoint, so these assertions are the baseline: any change to the
/// routing must change this file deliberately and visibly.
///
/// Each order asserts from inside its own calculate eval, via the `signer`
/// and `signed-context` words, exactly which signed context array it
/// received: the expected signer address and every content row. A failed
/// `ensure` reverts the whole clear, so a passing clear IS the routing
/// assertion.
contract RaindexV6ClearSignedContextTest is RaindexV6ExternalRealTest {
    using Strings for address;
    using Strings for uint256;

    address internal immutable iAlice = address(uint160(uint256(keccak256("alice.rain.test"))));
    address internal immutable iBob = address(uint160(uint256(keccak256("bob.rain.test"))));

    /// Private keys for the third party signers of alice's and bob's signed
    /// context. Distinct from the order owners so the tests distinguish "who
    /// signed it" from "whose side supplied it".
    uint256 internal constant ALICE_CONTEXT_SIGNER_KEY = 0xa11ce;
    uint256 internal constant BOB_CONTEXT_SIGNER_KEY = 0xb0b;

    bytes32 internal constant ALICE_INPUT_VAULT_ID = bytes32(uint256(0x10));
    bytes32 internal constant ALICE_OUTPUT_VAULT_ID = bytes32(uint256(0x20));
    bytes32 internal constant BOB_INPUT_VAULT_ID = bytes32(uint256(0x30));
    bytes32 internal constant BOB_OUTPUT_VAULT_ID = bytes32(uint256(0x40));

    /// Sign `context` as `LibContext.build` verifies it: an eth signed message
    /// over the keccak of the packed context words.
    function signContext(uint256 privateKey, bytes32[] memory context) internal pure returns (SignedContextV1 memory) {
        (uint8 v, bytes32 r, bytes32 s) =
            vm.sign(privateKey, MessageHashUtils.toEthSignedMessageHash(LibHashNoAlloc.hashWords(context)));
        return SignedContextV1({signer: vm.addr(privateKey), context: context, signature: abi.encodePacked(r, s, v)});
    }

    function singleSignedContext(uint256 privateKey, bytes32[] memory context)
        internal
        pure
        returns (SignedContextV1[] memory signedContexts)
    {
        signedContexts = new SignedContextV1[](1);
        signedContexts[0] = signContext(privateKey, context);
    }

    /// Two distinct, distinguishable content rows per side so a routing
    /// assertion can never pass on the wrong array.
    function aliceContextRows() internal pure returns (bytes32[] memory rows) {
        rows = new bytes32[](2);
        rows[0] = keccak256("alice.signed.context.0");
        rows[1] = keccak256("alice.signed.context.1");
    }

    function bobContextRows() internal pure returns (bytes32[] memory rows) {
        rows = new bytes32[](2);
        rows[0] = keccak256("bob.signed.context.0");
        rows[1] = keccak256("bob.signed.context.1");
    }

    /// `:ensure(...)` statements asserting that the eval running them sees
    /// exactly the signed context column signed by `expectedSigner` with
    /// `expectedRows` as its content.
    function buildSignedContextChecks(address expectedSigner, bytes32[] memory expectedRows, string memory label)
        internal
        pure
        returns (string memory checks)
    {
        checks =
            string.concat(":ensure(equal-to(signer<0>() ", expectedSigner.toHexString(), ") \"", label, " signer\")");
        for (uint256 i = 0; i < expectedRows.length; i++) {
            checks = string.concat(
                checks,
                ",:ensure(equal-to(signed-context<0 ",
                i.toString(),
                ">() ",
                uint256(expectedRows[i]).toHexString(),
                ") \"",
                label,
                " row ",
                i.toString(),
                "\")"
            );
        }
    }

    /// An order with constant calculate outputs whose calculate source runs
    /// `checks` in-band. Empty `checks` gives an order that never touches
    /// signed context.
    function buildRainString(string memory maxOutput, string memory ioRatio, string memory checks)
        internal
        view
        returns (bytes memory)
    {
        return bytes(
            string.concat(
                "using-words-from ",
                address(iSubParser).toHexString(),
                "\n_ _:",
                maxOutput,
                " ",
                ioRatio,
                bytes(checks).length == 0 ? "" : ",",
                checks,
                ";:;"
            )
        );
    }

    function deposit(address token, address owner, bytes32 vaultId) internal {
        vm.mockCall(
            token, abi.encodeWithSelector(IERC20.transferFrom.selector, owner, address(iRaindex)), abi.encode(true)
        );
        vm.prank(owner);
        iRaindex.deposit4(token, vaultId, LibDecimalFloat.packLossless(100, 0), new TaskV2[](0));
    }

    function addOrder(
        address owner,
        bytes memory rainString,
        address inputToken,
        bytes32 inputVaultId,
        address outputToken,
        bytes32 outputVaultId
    ) internal returns (OrderV4 memory) {
        IOV2[] memory validInputs = new IOV2[](1);
        validInputs[0] = IOV2({token: inputToken, vaultId: inputVaultId});
        IOV2[] memory validOutputs = new IOV2[](1);
        validOutputs[0] = IOV2({token: outputToken, vaultId: outputVaultId});
        OrderConfigV4 memory config = OrderConfigV4({
            evaluable: EvaluableV4({bytecode: iParserV2.parse2(rainString), interpreter: iInterpreter, store: iStore}),
            validInputs: validInputs,
            validOutputs: validOutputs,
            nonce: 0,
            secret: 0,
            meta: ""
        });
        vm.prank(owner);
        iRaindex.addOrder4(config, new TaskV2[](0));
        return OrderV4(owner, config.evaluable, config.validInputs, config.validOutputs, config.nonce);
    }

    /// Adds a funded alice order (token0 in, token1 out) with
    /// `aliceSignedContextChecks` in its calculate source and a funded
    /// matching bob order (token1 in, token0 out) with
    /// `bobSignedContextChecks` in its. Any failed in-eval check reverts the
    /// clear that evals the order.
    function setupClearOrders(string memory aliceSignedContextChecks, string memory bobSignedContextChecks)
        internal
        returns (OrderV4 memory orderAlice, OrderV4 memory orderBob)
    {
        orderAlice = addOrder(
            iAlice,
            buildRainString("5", "2", aliceSignedContextChecks),
            address(iToken0),
            ALICE_INPUT_VAULT_ID,
            address(iToken1),
            ALICE_OUTPUT_VAULT_ID
        );
        orderBob = addOrder(
            iBob,
            buildRainString("3", "0.5", bobSignedContextChecks),
            address(iToken1),
            BOB_INPUT_VAULT_ID,
            address(iToken0),
            BOB_OUTPUT_VAULT_ID
        );

        deposit(address(iToken1), iAlice, ALICE_OUTPUT_VAULT_ID);
        deposit(address(iToken0), iBob, BOB_OUTPUT_VAULT_ID);
    }

    /// `clear3` with default IO indexes and bounty vaults. The first external
    /// call after setup, so revert tests can `vm.expectRevert` immediately
    /// before it.
    function doClear(
        OrderV4 memory orderAlice,
        OrderV4 memory orderBob,
        SignedContextV1[] memory aliceSignedContext,
        SignedContextV1[] memory bobSignedContext
    ) internal {
        iRaindex.clear3(orderAlice, orderBob, ClearConfigV2(0, 0, 0, 0, 0, 0), aliceSignedContext, bobSignedContext);
    }

    /// Both sides provide distinct signed context: alice's eval sees the array
    /// supplied as `bobSignedContext` (bob's signer, bob's rows) and bob's
    /// eval sees the array supplied as `aliceSignedContext`.
    function testClear3SignedContextRoutedToCounterpartyEval() external {
        (OrderV4 memory orderAlice, OrderV4 memory orderBob) = setupClearOrders(
            buildSignedContextChecks(vm.addr(BOB_CONTEXT_SIGNER_KEY), bobContextRows(), "alice sees bob"),
            buildSignedContextChecks(vm.addr(ALICE_CONTEXT_SIGNER_KEY), aliceContextRows(), "bob sees alice")
        );
        doClear(
            orderAlice,
            orderBob,
            singleSignedContext(ALICE_CONTEXT_SIGNER_KEY, aliceContextRows()),
            singleSignedContext(BOB_CONTEXT_SIGNER_KEY, bobContextRows())
        );
    }

    /// Only alice provides signed context: it is delivered to bob's eval.
    /// Alice's eval receives the empty `bobSignedContext` so its expression
    /// must not read signed context at all; were the routing order-aligned,
    /// bob's `signer<0>` read would be out of bounds and revert the clear.
    function testClear3SignedContextOnlyAliceRoutedToBobEval() external {
        (OrderV4 memory orderAlice, OrderV4 memory orderBob) = setupClearOrders(
            "", buildSignedContextChecks(vm.addr(ALICE_CONTEXT_SIGNER_KEY), aliceContextRows(), "bob sees alice")
        );
        doClear(
            orderAlice,
            orderBob,
            singleSignedContext(ALICE_CONTEXT_SIGNER_KEY, aliceContextRows()),
            new SignedContextV1[](0)
        );
    }

    /// Only bob provides signed context: it is delivered to alice's eval.
    function testClear3SignedContextOnlyBobRoutedToAliceEval() external {
        (OrderV4 memory orderAlice, OrderV4 memory orderBob) = setupClearOrders(
            buildSignedContextChecks(vm.addr(BOB_CONTEXT_SIGNER_KEY), bobContextRows(), "alice sees bob"), ""
        );
        doClear(
            orderAlice,
            orderBob,
            new SignedContextV1[](0),
            singleSignedContext(BOB_CONTEXT_SIGNER_KEY, bobContextRows())
        );
    }

    /// The provider's own eval cannot see the context it supplied when the
    /// counterparty supplies none: alice's eval receives the empty
    /// `bobSignedContext`, so alice reading `signer<0>` is an out of bounds
    /// context read that reverts the clear even though `aliceSignedContext`
    /// holds exactly what the read asserts.
    function testClear3SignedContextProviderOwnEvalCannotReadIt() external {
        (OrderV4 memory orderAlice, OrderV4 memory orderBob) = setupClearOrders(
            buildSignedContextChecks(vm.addr(ALICE_CONTEXT_SIGNER_KEY), aliceContextRows(), "alice sees alice"), ""
        );
        vm.expectRevert(stdError.indexOOBError);
        doClear(
            orderAlice,
            orderBob,
            singleSignedContext(ALICE_CONTEXT_SIGNER_KEY, aliceContextRows()),
            new SignedContextV1[](0)
        );
    }

    /// Zero-length arrays on both sides build no signer or signed context
    /// columns at all: any `signer<0>` read is out of bounds and reverts the
    /// clear.
    function testClear3SignedContextEmptyBothSidesHasNoSignedColumns() external {
        (OrderV4 memory orderAlice, OrderV4 memory orderBob) =
            setupClearOrders(":ensure(equal-to(signer<0>() 0) \"unreachable\")", "");
        vm.expectRevert(stdError.indexOOBError);
        doClear(orderAlice, orderBob, new SignedContextV1[](0), new SignedContextV1[](0));
    }
}
