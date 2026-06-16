// SPDX-License-Identifier: LicenseRef-DCL-1.0
// SPDX-FileCopyrightText: Copyright (c) 2020 Rain Open Source Software Ltd
pragma solidity ^0.8.19;

import {ERC165, IERC165} from "openzeppelin-contracts/contracts/utils/introspection/ERC165.sol";
import {SafeERC20} from "openzeppelin-contracts/contracts/token/ERC20/utils/SafeERC20.sol";
import {IERC20} from "openzeppelin-contracts/contracts/token/ERC20/IERC20.sol";
import {ReentrancyGuard} from "openzeppelin-contracts/contracts/utils/ReentrancyGuard.sol";
import {ON_FLASH_LOAN_CALLBACK_SUCCESS} from "rain.raindex.interface/interface/ierc3156/IERC3156FlashBorrower.sol";
import {IRaindexV6, TakeOrdersConfigV5, TaskV2} from "rain.raindex.interface/interface/IRaindexV6.sol";
import {IERC3156FlashBorrower} from "rain.raindex.interface/interface/ierc3156/IERC3156FlashBorrower.sol";
import {OrderBookV6ArbCommon} from "./OrderBookV6ArbCommon.sol";
import {LibOrderBookArb} from "../lib/LibOrderBookArb.sol";
import {LibOrderBookDeploy} from "../lib/deploy/LibOrderBookDeploy.sol";
import {LibTOFUTokenDecimals} from "rain.tofu.erc20-decimals/lib/LibTOFUTokenDecimals.sol";
import {LibDecimalFloat} from "rain.math.float/lib/LibDecimalFloat.sol";

/// Thrown when the flash loan initiator is not this contract.
/// @param badInitiator The untrusted initiator of the flash loan.
error BadInitiator(address badInitiator);

/// Thrown when onFlashLoan is called by an address other than the deterministic
/// orderbook deployment. Prevents malicious contracts from invoking the callback.
/// @param badLender The untrusted caller of onFlashLoan.
error BadLender(address badLender);

/// Thrown when the flash loan fails somehow.
error FlashLoanFailed();

/// @title OrderBookV6FlashBorrower
/// @notice Abstract contract that liq-source specialized contracts can inherit
/// to provide flash loan based arbitrage against external liquidity sources to
/// fill orderbook orders.
///
/// For example consider a simple order:
///
/// input = DAI
/// output = USDT
/// IORatio = 1.01
/// Order amount = 100
///
/// Assume external liq is offering 102 DAI per USDT so it exceeds the IO ratio
/// but the order itself has no way to interact with the external contract.
/// The `OrderBookFlashBorrower` can:
///
/// - Flash loan 100 USDT from `Orderbook`
/// - Sell the 100 USDT for 102 DAI on external liq
/// - Take the order, giving 101 DAI and paying down 100 USDT loan
/// - Keep 1 DAI profit
///
/// As this contract is expected to be cloned using a minimal proxy there will
/// be many copies of it in the wild. Each copy can be access gated by a Rain
/// expression that is evaluated before anything else happens in `arb`. There
/// are many reasons why this might be desirable:
/// - Regulatory reasons that restrict how an arb bot operator can interact with
///   orders and/or external liq.
/// - The arb operator wants to attempt to prevent front running by other bots.
/// - The arb operator may prefer a dedicated instance of the contract to make
///   it easier to track profits, etc.
abstract contract OrderBookV6FlashBorrower is IERC3156FlashBorrower, ReentrancyGuard, ERC165, OrderBookV6ArbCommon {
    using SafeERC20 for IERC20;

    constructor() {}

    /// @inheritdoc IERC165
    function supportsInterface(bytes4 interfaceId) public view virtual override returns (bool) {
        return interfaceId == type(IERC3156FlashBorrower).interfaceId || super.supportsInterface(interfaceId);
    }

    /// @dev Hook that inheriting contracts SHOULD override in order to achieve
    /// anything other than raising the ambient temperature of the room.
    /// `_exchange` is responsible for converting the flash loaned assets into
    /// the assets required to fill the orders. Generally this can only be
    /// achieved by interacting with an external liquidity source that is
    /// offering a better price than the orders require.
    /// @param takeOrders As per `arb`.
    /// @param exchangeData As per `arb`.
    //slither-disable-next-line dead-code
    function _exchange(TakeOrdersConfigV5 memory takeOrders, bytes memory exchangeData) internal virtual {}

    /// @inheritdoc IERC3156FlashBorrower
    function onFlashLoan(address initiator, address, uint256, uint256, bytes calldata data) external returns (bytes32) {
        // Only the deterministic orderbook deployment may call this callback.
        if (msg.sender != LibOrderBookDeploy.ORDERBOOK_DEPLOYED_ADDRESS) {
            revert BadLender(msg.sender);
        }
        // As per reference implementation.
        if (initiator != address(this)) {
            revert BadInitiator(initiator);
        }

        (TakeOrdersConfigV5 memory takeOrders, bytes memory exchangeData) =
            abi.decode(data, (TakeOrdersConfigV5, bytes));

        // Dispatch the `_exchange` hook to ensure we have the correct asset
        // type and amount to fill the orders.
        _exchange(takeOrders, exchangeData);

        // At this point `_exchange` should have sent the tokens required to match
        // the orders so take orders now.
        // We don't do anything with the total input/output amounts here because
        // the flash loan itself will take back what it needs, and we simply
        // keep anything left over according to active balances.
        //slither-disable-next-line unused-return
        IRaindexV6(msg.sender).takeOrders4(takeOrders);

        return ON_FLASH_LOAN_CALLBACK_SUCCESS;
    }

    /// Primary function to process arbitrage opportunities.
    /// The configured task hash is validated via `onlyValidTask` to gate access.
    /// If there is no task configured, anyone can call this and should expect
    /// to be front run on the arb for any sufficiently profitable opportunity.
    ///
    /// A flash loan is taken from the orderbook, then inside the callback
    /// `_exchange` converts the borrowed tokens into the tokens needed to fill
    /// the orders, and the orders are taken. After the flash loan repays,
    /// remaining assets are sent to the sender via `finalizeArb`.
    ///
    /// @param orderBook The `OrderBook` contract to take orders from and flash
    /// loan against.
    /// @param takeOrders As per `IRaindexV6.takeOrders4`.
    /// @param exchangeData Arbitrary bytes that will be passed to `_exchange`
    /// after the flash loan is taken. The inheriting contract is responsible
    /// for decoding this data and defining how it controls interactions with
    /// the external liquidity. For example, `GenericPoolOrderBookV6FlashBorrower`
    /// uses this data as a literal encoded external call.
    /// @param task The task to evaluate after the arb completes.
    function arb4(
        IRaindexV6 orderBook,
        TakeOrdersConfigV5 calldata takeOrders,
        bytes calldata exchangeData,
        TaskV2 calldata task
    ) external payable nonReentrant {
        _beforeArb(task);
        // Mimic what OB would do anyway if called with zero orders.
        if (takeOrders.orders.length == 0) {
            revert IRaindexV6.NoOrders();
        }

        // Encode everything that will be used by the flash loan callback.
        bytes memory data = abi.encode(takeOrders, exchangeData);
        // The token we receive from taking the orders is what we will use to
        // repay the flash loan.
        address ordersOutputToken = takeOrders.orders[0].order.validOutputs[takeOrders.orders[0].outputIOIndex].token;
        address ordersInputToken = takeOrders.orders[0].order.validInputs[takeOrders.orders[0].inputIOIndex].token;

        uint8 inputDecimals = LibTOFUTokenDecimals.safeDecimalsForToken(ordersInputToken);
        uint8 outputDecimals = LibTOFUTokenDecimals.safeDecimalsForToken(ordersOutputToken);

        // We can't repay more than the minimum that the orders are going to
        // give us and there's no reason to borrow less.
        uint256 flashLoanAmount = LibDecimalFloat.toFixedDecimalLossless(takeOrders.minimumIO, outputDecimals);

        // Take the flash loan, which will in turn call `onFlashLoan`, which is
        // expected to process an exchange against external liq to pay back the
        // flash loan, cover the orders and remain in profit.
        IERC20(ordersInputToken).forceApprove(address(orderBook), type(uint256).max);
        IERC20(ordersOutputToken).forceApprove(address(orderBook), type(uint256).max);
        if (!orderBook.flashLoan(this, ordersOutputToken, flashLoanAmount, data)) {
            revert FlashLoanFailed();
        }
        IERC20(ordersOutputToken).forceApprove(address(orderBook), 0);
        IERC20(ordersInputToken).forceApprove(address(orderBook), 0);

        LibOrderBookArb.finalizeArb(task, ordersInputToken, inputDecimals, ordersOutputToken, outputDecimals);
    }
}
