// SPDX-License-Identifier: LicenseRef-DCL-1.0
// SPDX-FileCopyrightText: Copyright (c) 2020 Rain Open Source Software Ltd
pragma solidity =0.8.25;

import {IERC20} from "@openzeppelin-contracts-5.6.1/token/ERC20/IERC20.sol";
import {RaindexV6ExternalRealTest} from "test/util/abstract/RaindexV6ExternalRealTest.sol";
import {LibTestTakeOrder} from "test/util/lib/LibTestTakeOrder.sol";
import {
    OrderV4,
    TakeOrderConfigV4,
    TakeOrdersConfigV5,
    SignedContextV1,
    TaskV2
} from "raindex-interface-0.1.1/src/interface/IRaindexV6.sol";
import {Float, LibDecimalFloat} from "rain-math-float-0.1.1/src/lib/LibDecimalFloat.sol";

/// @title RaindexV6TakeOrderBatchEvalConsistencyTest
/// @notice Within a single `takeOrders4` batch, each order's `calculate` should
/// see the complete effects of every earlier order in the batch — a sequential
/// view. Today that holds for some state and not others, depending purely on how
/// the effect is applied:
///
///   - internal vault balance change -> applied in-loop (SSTORE)        -> VISIBLE
///   - vault-0 input credit          -> token push deferred past loop   -> NOT visible
///   - earlier order's calculate set -> threaded into later calculate   -> VISIBLE
///   - earlier order's handle-IO set -> committed after the loop         -> NOT visible
///
/// Each pair below uses two same-owner orders: order A fills and produces an
/// effect; order B reads that effect in `calculate` and only fills while it has
/// NOT observed it (`if(seen 0 1)`). So a fully sequential batch fills exactly
/// one order (total 1 — B observes A and skips); an inconsistent one fills both
/// (total 2 — B is blind to A). The internal-vault and calculate-write cases
/// pass today; the vault-0 and handle-IO cases fail, pinning the two seams.
contract RaindexV6TakeOrderBatchEvalConsistencyTest is RaindexV6ExternalRealTest {
    using LibDecimalFloat for Float;

    address internal immutable iTaker = address(uint160(uint256(keccak256("taker.rain.test"))));

    bytes32 internal constant INTERNAL_VAULT = bytes32(uint256(0x01));
    bytes32 internal constant VAULT_ZERO = bytes32(0);

    function owner(string memory tag) internal pure returns (address) {
        return address(uint160(uint256(keccak256(bytes(tag)))));
    }

    /// Allow every settlement transfer to succeed (deposits, taker legs, and the
    /// deferred vault-0 input push to the owner).
    function mockTransfers() internal {
        vm.mockCall(address(iToken0), abi.encodeWithSelector(IERC20.transfer.selector), abi.encode(true));
        vm.mockCall(address(iToken0), abi.encodeWithSelector(IERC20.transferFrom.selector), abi.encode(true));
        vm.mockCall(address(iToken1), abi.encodeWithSelector(IERC20.transfer.selector), abi.encode(true));
        vm.mockCall(address(iToken1), abi.encodeWithSelector(IERC20.transferFrom.selector), abi.encode(true));
    }

    /// Vault-0 input reads the owner's live wallet balance/allowance for the
    /// input token; mock both to zero so the only thing that could make a later
    /// order see a balance is an earlier order's (currently deferred) credit.
    function mockVaultZeroReads(address o) internal {
        vm.mockCall(address(iToken0), abi.encodeWithSelector(IERC20.balanceOf.selector, o), abi.encode(0));
        vm.mockCall(
            address(iToken0), abi.encodeWithSelector(IERC20.allowance.selector, o, address(iRaindex)), abi.encode(0)
        );
    }

    /// Fund the owner's iToken1 output vault so orders have output to give.
    function fundOutput(address o) internal {
        vm.prank(o);
        iRaindex.deposit4(address(iToken1), INTERNAL_VAULT, LibDecimalFloat.packLossless(100, 0), new TaskV2[](0));
    }

    function addOrder(address o, bytes memory expr, bytes32 inputVaultId) internal returns (OrderV4 memory) {
        return LibTestTakeOrder.addOrderWithExpression(
            vm, o, expr, address(iToken0), inputVaultId, address(iToken1), INTERNAL_VAULT
        );
    }

    function take(OrderV4 memory a, OrderV4 memory b) internal returns (Float totalTakerInput) {
        TakeOrderConfigV4[] memory orders = new TakeOrderConfigV4[](2);
        orders[0] =
            TakeOrderConfigV4({order: a, inputIOIndex: 0, outputIOIndex: 0, signedContext: new SignedContextV1[](0)});
        orders[1] =
            TakeOrderConfigV4({order: b, inputIOIndex: 0, outputIOIndex: 0, signedContext: new SignedContextV1[](0)});
        vm.prank(iTaker);
        (totalTakerInput,) = iRaindex.takeOrders4(LibTestTakeOrder.defaultTakeConfig(orders));
    }

    // A: plain fill of 1, crediting its input vault.
    bytes constant FILL = "_ _:1 1;:;";
    // B: fill 1 only while its input vault balance is still zero. `context<3 3>`
    // is the input vault balance before the trade (the `input-vault-before` word).
    bytes constant FILL_WHILE_INPUT_EMPTY = "bal:context<3 3>(),_ _:if(bal 0 1) 1;:;";

    /// Baseline (consistent): an internal-vault input credit from A is visible to
    /// B's calculate in the same batch, so B observes it and skips. Total 1.
    function testInternalInputBalanceVisibleToLaterSameOwnerOrder() external {
        mockTransfers();
        address o = owner("alice");
        fundOutput(o);
        OrderV4 memory a = addOrder(o, FILL, INTERNAL_VAULT);
        OrderV4 memory b = addOrder(o, FILL_WHILE_INPUT_EMPTY, INTERNAL_VAULT);
        assertTrue(take(a, b).eq(LibDecimalFloat.packLossless(1, 0)), "B must see A's internal input credit and skip");
    }

    /// Baseline (consistent): A's calculate `set` is threaded into B's calculate
    /// in the same batch, so B observes it and skips. Total 1.
    function testCalculateWriteVisibleToLaterSameOwnerOrder() external {
        mockTransfers();
        address o = owner("bob");
        fundOutput(o);
        OrderV4 memory a = addOrder(o, ":set(0 1),_ _:1 1;:;", INTERNAL_VAULT);
        OrderV4 memory b = addOrder(o, "seen:get(0),_ _:if(seen 0 1) 1;:;", INTERNAL_VAULT);
        assertTrue(take(a, b).eq(LibDecimalFloat.packLossless(1, 0)), "B must see A's calculate write and skip");
    }

    /// Inconsistent (FAILS today): A's vault-0 input credit is a token push
    /// deferred past the loop, so B's calculate reads a zero balance and fills.
    /// Identical to the internal-vault baseline except the input vault id, which
    /// is the whole point: the visibility must not depend on the vault id.
    function testVaultZeroInputBalanceVisibleToLaterSameOwnerOrder() external {
        mockTransfers();
        address o = owner("carol");
        mockVaultZeroReads(o);
        fundOutput(o);
        OrderV4 memory a = addOrder(o, FILL, VAULT_ZERO);
        OrderV4 memory b = addOrder(o, FILL_WHILE_INPUT_EMPTY, VAULT_ZERO);
        assertTrue(take(a, b).eq(LibDecimalFloat.packLossless(1, 0)), "B must see A's vault-0 input credit and skip");
    }

    /// Inconsistent (FAILS today): A's handle-IO `set` is committed after the
    /// loop, so B's calculate reads a zero value and fills. Identical to the
    /// calculate-write baseline except the `set` is in handle-IO rather than
    /// calculate.
    function testHandleIOWriteVisibleToLaterSameOwnerOrder() external {
        mockTransfers();
        address o = owner("dave");
        fundOutput(o);
        OrderV4 memory a = addOrder(o, "_ _:1 1;:set(0 1);", INTERNAL_VAULT);
        OrderV4 memory b = addOrder(o, "seen:get(0),_ _:if(seen 0 1) 1;:;", INTERNAL_VAULT);
        assertTrue(take(a, b).eq(LibDecimalFloat.packLossless(1, 0)), "B must see A's handle-IO write and skip");
    }
}
