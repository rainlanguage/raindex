// SPDX-License-Identifier: LicenseRef-DCL-1.0
// SPDX-FileCopyrightText: Copyright (c) 2020 Rain Open Source Software Ltd
pragma solidity =0.8.25;

import {Test} from "forge-std-1.16.1/src/Test.sol";
import {IERC20} from "@openzeppelin-contracts-5.6.1/token/ERC20/IERC20.sol";
import {MockToken} from "test/util/concrete/MockToken.sol";
import {LibGenericPoolExchange} from "src/lib/LibGenericPoolExchange.sol";

/// @dev Harness exposing the internal `exchange` for direct testing.
contract LibGenericPoolExchangeHarness {
    function exchange(address token, bytes memory data) external {
        LibGenericPoolExchange.exchange(token, data);
    }
}

/// @dev Invoked as the `pool`; records, at call time, the caller and the
/// caller's token allowance to both `spender` and the pool itself — so a test
/// can prove the approval went to `spender` (not the pool) and the call routed
/// here.
contract AllowanceProbePool {
    address public caller;
    uint256 public spenderAllowance;
    uint256 public poolAllowance;

    function probe(IERC20 token, address arb, address spender) external payable {
        caller = msg.sender;
        spenderAllowance = token.allowance(arb, spender);
        poolAllowance = token.allowance(arb, address(this));
    }
}

/// @dev Accepts any call (including empty calldata) so the empty-encoded-call
/// path can be observed.
contract FallbackPool {
    bool public called;

    fallback() external payable {
        called = true;
    }

    receive() external payable {
        called = true;
    }
}

/// @title LibGenericPoolExchangeSpenderPoolTest
/// @notice Audit A06-1/A06-2 (#2533): existing tests always set spender == pool.
/// Covers the spender != pool path (approve `spender`, call `pool`, revoke) and
/// the empty-encoded-call decode edge.
contract LibGenericPoolExchangeSpenderPoolTest is Test {
    LibGenericPoolExchangeHarness internal immutable iHarness = new LibGenericPoolExchangeHarness();

    /// A06-1: with spender != pool, the approval is granted to `spender` (not the
    /// pool) for the duration of the call to `pool`, then revoked.
    function testSpenderNotPoolApprovesSpenderCallsPool(address spender) external {
        MockToken token = new MockToken("Token", "TKN", 18);
        AllowanceProbePool pool = new AllowanceProbePool();
        vm.assume(spender != address(0));
        vm.assume(spender != address(pool));
        vm.assume(spender != address(token));

        bytes memory call =
            abi.encodeCall(AllowanceProbePool.probe, (IERC20(address(token)), address(iHarness), spender));
        iHarness.exchange(address(token), abi.encode(spender, address(pool), call));

        assertEq(pool.caller(), address(iHarness), "call routed to pool from the exchanger");
        assertEq(pool.spenderAllowance(), type(uint256).max, "spender approved for the call");
        assertEq(pool.poolAllowance(), 0, "pool itself NOT approved (spender != pool)");
        assertEq(token.allowance(address(iHarness), spender), 0, "approval revoked after the call");
    }

    /// A06-2: an empty `encodedFunctionCall` is still forwarded to the pool
    /// (hitting its receive/fallback) and the approve/revoke still runs.
    function testEmptyEncodedCallForwardedToPool(address spender) external {
        MockToken token = new MockToken("Token", "TKN", 18);
        FallbackPool pool = new FallbackPool();
        vm.assume(spender != address(0));

        iHarness.exchange(address(token), abi.encode(spender, address(pool), bytes("")));

        assertTrue(pool.called(), "empty call still reaches the pool");
        assertEq(token.allowance(address(iHarness), spender), 0, "approval revoked after the call");
    }
}
