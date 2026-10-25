// SPDX-License-Identifier: LicenseRef-DCL-1.0
// SPDX-FileCopyrightText: Copyright (c) 2020 Rain Open Source Software Ltd
pragma solidity ^0.8.19;

import {ERC165, IERC165} from "@openzeppelin-contracts-5.7.0/utils/introspection/ERC165.sol";
import {ReentrancyGuard} from "@openzeppelin-contracts-5.7.0/utils/ReentrancyGuard.sol";
import {IERC20, SafeERC20} from "@openzeppelin-contracts-5.7.0/token/ERC20/utils/SafeERC20.sol";
import {IRaindexV6} from "raindex-interface-0.1.8/src/interface/IRaindexV6.sol";
import {IRaindexV6ArbOrderTaker, TaskV2} from "raindex-interface-0.1.8/src/interface/IRaindexV6ArbOrderTaker.sol";
import {TakeOrdersConfigV5, Float} from "raindex-interface-0.1.8/src/interface/IRaindexV6.sol";
import {RaindexV6ArbCommon} from "./RaindexV6ArbCommon.sol";
import {LibRaindexArb} from "../lib/LibRaindexArb.sol";
import {LibRaindexDeploy} from "../lib/deploy/LibRaindexDeploy.sol";
import {IRaindexV6OrderTaker} from "raindex-interface-0.1.8/src/interface/IRaindexV6OrderTaker.sol";
import {LibTOFUTokenDecimals} from "rain-tofu-erc20-decimals-0.1.4/src/lib/LibTOFUTokenDecimals.sol";

/// Thrown when `arb5` is called with a `raindex` that is not the trusted
/// deterministic deployment. `arb5` grants `raindex` an unlimited token approval
/// and calls it, so an untrusted address could drain the contract's balance.
error BadRaindex(address badRaindex);

/// Thrown when `onTakeOrders2` is called during an arb by anything other than
/// the trusted raindex.
error UnexpectedOnTakeOrdersCaller(address caller);

/// @title RaindexV6ArbOrderTaker
/// @notice Arb contract that takes orders directly from a `Raindex` without
/// flash loans. Inheritors implement the strategy for sourcing the input token
/// (e.g. routing through a DEX).
abstract contract RaindexV6ArbOrderTaker is
    IRaindexV6OrderTaker,
    IRaindexV6ArbOrderTaker,
    ReentrancyGuard,
    ERC165,
    RaindexV6ArbCommon
{
    using SafeERC20 for IERC20;

    constructor() {}

    /// @inheritdoc IERC165
    function supportsInterface(bytes4 interfaceId) public view virtual override returns (bool) {
        return (interfaceId == type(IRaindexV6OrderTaker).interfaceId)
            || (interfaceId == type(IRaindexV6ArbOrderTaker).interfaceId) || super.supportsInterface(interfaceId);
    }

    /// @inheritdoc IRaindexV6ArbOrderTaker
    function arb5(IRaindexV6 raindex, TakeOrdersConfigV5 calldata takeOrders, TaskV2 calldata task)
        external
        payable
        nonReentrant
    {
        _beforeArb(task);
        // Mimic what Raindex would do anyway if called with zero orders.
        if (takeOrders.orders.length == 0) {
            revert IRaindexV6.NoOrders();
        }
        // The arb grants `raindex` an unlimited token approval and calls it, so
        // it MUST be the trusted deployment rather than a caller-supplied address.
        if (address(raindex) != LibRaindexDeploy.RAINDEX_DEPLOYED_ADDRESS) {
            revert BadRaindex(address(raindex));
        }

        address ordersInputToken = takeOrders.orders[0].order.validInputs[takeOrders.orders[0].inputIOIndex].token;
        address ordersOutputToken = takeOrders.orders[0].order.validOutputs[takeOrders.orders[0].outputIOIndex].token;

        IERC20(ordersInputToken).forceApprove(address(raindex), type(uint256).max);
        //slither-disable-next-line unused-return
        raindex.takeOrders4(takeOrders);
        IERC20(ordersInputToken).forceApprove(address(raindex), 0);

        LibRaindexArb.finalizeArb(
            task,
            ordersInputToken,
            LibTOFUTokenDecimals.safeDecimalsForToken(ordersInputToken),
            ordersOutputToken,
            LibTOFUTokenDecimals.safeDecimalsForToken(ordersOutputToken)
        );
    }

    /// @inheritdoc IRaindexV6OrderTaker
    /// @dev While `arb5` is in flight only the trusted raindex may call this;
    /// any other caller reverts with `UnexpectedOnTakeOrdersCaller`. During an
    /// arb the contract holds the caller's `msg.value` and tokens, and
    /// `takeOrders4` runs order-owner-chosen code (e.g. the order's store) that
    /// could otherwise re-enter here and direct that value.
    ///
    /// Outside an arb it is a permissionless no-op: the contract holds no value
    /// between operations, so there is nothing to protect. The permissionless
    /// entry is the recovery path for value that reaches the contract by
    /// accident. Unsolicited transfers cannot be prevented: ERC20 transfers have
    /// no recipient hook to revert in, and the arb legitimately receives ERC20
    /// and ETH mid-operation. A stray balance is swept out by the next call
    /// rather than stranded. Concrete implementations that move value here (e.g.
    /// the generic-pool variant's unlimited approve-and-call with whole-ETH-
    /// balance forward) rely on both properties.
    function onTakeOrders2(address, address, Float, Float, bytes calldata) public virtual override {
        if (_reentrancyGuardEntered() && msg.sender != LibRaindexDeploy.RAINDEX_DEPLOYED_ADDRESS) {
            revert UnexpectedOnTakeOrdersCaller(msg.sender);
        }
    }
}
