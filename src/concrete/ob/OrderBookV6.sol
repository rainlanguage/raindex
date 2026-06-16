// SPDX-License-Identifier: LicenseRef-DCL-1.0
// SPDX-FileCopyrightText: Copyright (c) 2020 Rain Open Source Software Ltd
pragma solidity =0.8.25;

import {Math} from "openzeppelin-contracts/contracts/utils/math/Math.sol";
import {Multicall} from "openzeppelin-contracts/contracts/utils/Multicall.sol";
import {IERC20} from "openzeppelin-contracts/contracts/token/ERC20/IERC20.sol";
import {SafeERC20} from "openzeppelin-contracts/contracts/token/ERC20/utils/SafeERC20.sol";
import {ReentrancyGuard} from "openzeppelin-contracts/contracts/utils/ReentrancyGuard.sol";
import {IERC20Metadata} from "openzeppelin-contracts/contracts/token/ERC20/extensions/IERC20Metadata.sol";

import {LibContext} from "rain.interpreter.interface/lib/caller/LibContext.sol";
import {LibBytecode} from "rain.interpreter.interface/lib/bytecode/LibBytecode.sol";
import {
    SourceIndexV2,
    StateNamespace,
    IInterpreterV4,
    StackItem,
    EvalV4
} from "rain.interpreter.interface/interface/IInterpreterV4.sol";
import {LibUint256Array} from "rain.solmem/lib/LibUint256Array.sol";
import {LibUint256Matrix} from "rain.solmem/lib/LibUint256Matrix.sol";
import {IInterpreterStoreV3} from "rain.interpreter.interface/interface/IInterpreterStoreV3.sol";
import {LibNamespace} from "rain.interpreter.interface/lib/ns/LibNamespace.sol";
import {LibMeta} from "rain.metadata/lib/LibMeta.sol";
import {IMetaV1_2} from "rain.metadata/interface/unstable/IMetaV1_2.sol";
import {LibOrderBook} from "../../lib/LibOrderBook.sol";
import {LibDecimalFloat} from "rain.math.float/lib/LibDecimalFloat.sol";
import {LibTOFUTokenDecimals, TOFUOutcome} from "rain.tofu.erc20-decimals/lib/LibTOFUTokenDecimals.sol";
import {ITOFUTokenDecimals} from "rain.tofu.erc20-decimals/interface/ITOFUTokenDecimals.sol";

import {
    IRaindexV6,
    OrderV4,
    OrderConfigV4,
    TakeOrderConfigV4,
    TakeOrdersConfigV5,
    ClearConfigV2,
    ClearStateChangeV2,
    SignedContextV1,

    //forge-lint: disable-next-line(unused-import)
    EvaluableV4,
    TaskV2,
    QuoteV2,
    Float,
    IOV2
} from "rain.raindex.interface/interface/IRaindexV6.sol";
import {IRaindexV6OrderTaker} from "rain.raindex.interface/interface/IRaindexV6OrderTaker.sol";
import {LibOrder} from "../../lib/LibOrder.sol";
import {
    CALLING_CONTEXT_COLUMNS,
    CONTEXT_CALLING_CONTEXT_COLUMN,
    CONTEXT_CALCULATIONS_COLUMN,
    CONTEXT_VAULT_IO_BALANCE_DIFF,
    CONTEXT_VAULT_INPUTS_COLUMN,
    CONTEXT_VAULT_IO_TOKEN,
    CONTEXT_VAULT_OUTPUTS_COLUMN,
    CONTEXT_VAULT_IO_VAULT_ID
} from "../../lib/LibOrderBook.sol";
import {OrderBookV6FlashLender} from "../../abstract/OrderBookV6FlashLender.sol";
import {LibBytes32Array} from "rain.solmem/lib/LibBytes32Array.sol";
import {LibBytes32Matrix} from "rain.solmem/lib/LibBytes32Matrix.sol";

import {LibFormatDecimalFloat} from "rain.math.float/lib/format/LibFormatDecimalFloat.sol";

/// This will exist in a future version of Open Zeppelin if their main branch is
/// to be believed.
error ReentrancyGuardReentrantCall();

/// Thrown when the `msg.sender` modifying an order is not its owner.
/// @param owner The owner of the order.
error NotOrderOwner(address owner);

/// Thrown when the input and output tokens don't match, in either direction.
error TokenMismatch();

/// Thrown when the input token is the output token.
error TokenSelfTrade();

/// Thrown when the minimum input is not met.
/// @param minimumIO The minimum io required.
/// @param actualIO The io that was achieved.
error MinimumIO(Float minimumIO, Float actualIO);

/// Thrown when two orders have the same owner during clear.
error SameOwner();

/// Thrown when calculate order expression offers too few outputs.
/// @param outputs The outputs the expression offers.
error UnsupportedCalculateOutputs(uint256 outputs);

/// Thrown when a negative vault balance is being recorded.
/// @param vaultBalance The negative vault balance being recorded.
error NegativeVaultBalance(Float vaultBalance);

/// Thrown when a negative amount is being applied to a vault balance.
/// @param amount The negative amount being applied.
error NegativeVaultBalanceChange(Float amount);

/// Thrown when a negative pull is attempted.
error NegativePull();

/// Thrown when a negative push is attempted.
error NegativePush();

/// Throws when a negative bounty is calculated.
error NegativeBounty();

/// Thrown when clear output amounts are both zero.
error ClearZeroAmount();

/// @dev Stored value for a live order. NOT a boolean because storing a boolean
/// is more expensive than storing a uint256.
uint256 constant ORDER_LIVE = 1;

/// @dev Stored value for a dead order. `0` is chosen because it is the default
/// value for a mapping, which means all orders are dead unless explicitly made
/// live.
uint256 constant ORDER_DEAD = 0;

/// @dev Entrypoint to a calculate the amount and ratio of an order.
SourceIndexV2 constant CALCULATE_ORDER_ENTRYPOINT = SourceIndexV2.wrap(0);
/// @dev Entrypoint to handle the final internal vault movements resulting from
/// matching multiple calculated orders.
SourceIndexV2 constant HANDLE_IO_ENTRYPOINT = SourceIndexV2.wrap(1);

/// @dev Minimum outputs for calculate order are the amount and ratio.
uint256 constant CALCULATE_ORDER_MIN_OUTPUTS = 2;
/// @dev Maximum outputs for calculate order are the amount and ratio.
uint16 constant CALCULATE_ORDER_MAX_OUTPUTS = 2;

/// @dev Handle IO has no outputs as it only responds to vault movements.
uint256 constant HANDLE_IO_MIN_OUTPUTS = 0;
/// @dev Handle IO has no outputs as it only responds to vault movements.
uint16 constant HANDLE_IO_MAX_OUTPUTS = 0;

/// All information resulting from an order calculation that allows for vault IO
/// to be calculated and applied, then the handle IO entrypoint to be dispatched.
/// @param outputMax The maximum output calculated by the order expression as a
/// Float. WILL BE RESCALED ACCORDING TO TOKEN DECIMALS (read via TOFU) for the
/// purpose of calculating actual vault movements.
/// The output max is CAPPED AT THE OUTPUT VAULT BALANCE OF THE ORDER OWNER.
/// The order is guaranteed that the total output of this single clearance cannot
/// exceed this (subject to rescaling). It is up to the order expression to track
/// values over time if the output max is to impose a global limit across many
/// transactions and counterparties.
/// @param IORatio The order ratio as input/output from the perspective of the
/// order, as a Float. As each counterparty's input is the other's output, the
/// IORatio calculated by each order is inverse of its counterparty. IORatio is
/// RESCALED ACCORDING TO TOKEN DECIMALS (read via TOFU) when applied to vault
/// balances. I.e. a ratio of `1` is ALWAYS "one" as ECONOMIC EQUIVALENCE
/// between two tokens, but this will be rescaled according to the decimals of
/// the token. For example, if DAI and USDT have a ratio of `1` then in reality
/// `1e12` DAI will move in the vault for every `1` USDT that moves, because DAI
/// has `1e18` decimals per $1 peg and USDT has `1e6` decimals per $1 peg.
/// @param context The entire 2D context array, initialized from the context
/// passed into the order calculations and then populated with the order
/// calculations and vault IO before being passed back to handle IO entrypoint.
/// @param namespace The `StateNamespace` to be passed to the store for calculate
/// IO state changes.
/// @param kvs KVs returned from calculate order entrypoint to pass to the store
/// before calling handle IO entrypoint.
struct OrderIOCalculationV4 {
    OrderV4 order;
    uint256 outputIOIndex;
    Float outputMax;
    //solhint-disable-next-line var-name-mixedcase
    Float IORatio;
    bytes32[][] context;
    StateNamespace namespace;
    bytes32[] kvs;
}

/// @title OrderBookV6
/// See `IRaindexV6` for more documentation.
contract OrderBookV6 is IRaindexV6, IMetaV1_2, ReentrancyGuard, Multicall, OrderBookV6FlashLender {
    using LibUint256Array for uint256[];
    using SafeERC20 for IERC20;
    using LibOrder for OrderV4;
    using LibUint256Array for uint256;
    using LibDecimalFloat for Float;
    using LibBytes32Array for bytes32;

    /// All hashes of all active orders. There's nothing interesting in the value
    /// it's just nonzero if the order is live. The key is the hash of the order.
    /// Removing an order sets the value back to zero so it is identical to the
    /// order never existing.
    /// The order hash includes its owner so there's no need to build a multi
    /// level mapping, each order hash MUST uniquely identify the order globally.
    /// order hash => order is live
    // Solhint and slither disagree on this. Slither wins.
    //solhint-disable-next-line private-vars-leading-underscore
    mapping(bytes32 orderHash => uint256 liveness) internal sOrders;

    /// @dev Vault balances are stored in a mapping of owner => token => vault ID
    /// This gives 1:1 parity with the `IRaindexV6` interface but keeping the
    /// `sFoo` naming convention for storage variables.
    // Solhint and slither disagree on this. Slither wins.
    //solhint-disable-next-line private-vars-leading-underscore
    mapping(address owner => mapping(address token => mapping(bytes32 vaultId => Float balance))) internal
        sVaultBalances;

    /// @inheritdoc IRaindexV6
    function vaultBalance2(address owner, address token, bytes32 vaultId) external view override returns (Float) {
        return _vaultBalance(owner, token, vaultId);
    }

    /// @dev Returns the effective vault balance. For vault ID 0, returns the
    /// minimum of the owner's token balance and their approval to this contract.
    //slither-disable-next-line calls-loop
    function _vaultBalance(address owner, address token, bytes32 vaultId) internal view returns (Float) {
        if (vaultId != bytes32(0)) {
            return sVaultBalances[owner][token][vaultId];
        } else {
            (TOFUOutcome tofuOutcome, uint8 decimals) = LibTOFUTokenDecimals.decimalsForTokenReadOnly(token);
            if (tofuOutcome != TOFUOutcome.Consistent && tofuOutcome != TOFUOutcome.Initial) {
                revert ITOFUTokenDecimals.TokenDecimalsReadFailure(token, tofuOutcome);
            }
            Float ownerTokenBalance =
                LibDecimalFloat.fromFixedDecimalLosslessPacked(IERC20(token).balanceOf(owner), decimals);
            //slither-disable-next-line unused-return
            (Float ownerTokenApproval,) =
                LibDecimalFloat.fromFixedDecimalLossyPacked(IERC20(token).allowance(owner, address(this)), decimals);
            return ownerTokenBalance.min(ownerTokenApproval);
        }
    }

    /// @inheritdoc IRaindexV6
    function orderExists(bytes32 orderHash) external view override returns (bool) {
        return sOrders[orderHash] == ORDER_LIVE;
    }

    /// @inheritdoc IRaindexV6
    function entask2(TaskV2[] calldata post) external nonReentrant {
        LibOrderBook.doPost(new bytes32[][](0), post);
    }

    /// @inheritdoc IRaindexV6
    function deposit4(address token, bytes32 vaultId, Float depositAmount, TaskV2[] calldata post)
        external
        nonReentrant
        nonZeroVaultId(msg.sender, token, vaultId)
    {
        if (!depositAmount.gt(Float.wrap(0))) {
            revert ZeroDepositAmount(msg.sender, token, vaultId);
        }

        (uint256 depositAmountUint256, uint8 decimals) = pullTokens(msg.sender, token, depositAmount);

        // It is safest with vault deposits to move tokens in to the Orderbook
        // before updating internal vault balances although we have a reentrancy
        // guard in place anyway.
        emit DepositV2(msg.sender, token, vaultId, depositAmountUint256);

        (Float beforeBalance, Float afterBalance) = increaseVaultBalance(msg.sender, token, vaultId, depositAmount);

        if (post.length != 0) {
            LibOrderBook.doPost(
                LibBytes32Matrix.matrixFrom(
                    LibBytes32Array.arrayFrom(
                        bytes32(uint256(uint160(token))),
                        bytes32(vaultId),
                        Float.unwrap(beforeBalance),
                        Float.unwrap(afterBalance),
                        bytes32(uint256(decimals))
                    )
                ),
                post
            );
        }
    }

    /// @inheritdoc IRaindexV6
    function withdraw4(address token, bytes32 vaultId, Float targetAmount, TaskV2[] calldata post)
        external
        nonReentrant
        nonZeroVaultId(msg.sender, token, vaultId)
    {
        Float withdrawAmount;
        uint8 decimals;
        Float beforeBalance;
        Float afterBalance;
        {
            if (!targetAmount.gt(Float.wrap(0))) {
                revert ZeroWithdrawTargetAmount(msg.sender, token, vaultId);
            }
            Float currentVaultBalance = _vaultBalance(msg.sender, token, vaultId);
            withdrawAmount = targetAmount.min(currentVaultBalance);
            (beforeBalance, afterBalance) = decreaseVaultBalance(msg.sender, token, vaultId, withdrawAmount);

            uint256 withdrawAmountUint256;
            (withdrawAmountUint256, decimals) = pushTokens(msg.sender, token, withdrawAmount);
            emit WithdrawV2(msg.sender, token, vaultId, targetAmount, withdrawAmount, withdrawAmountUint256);
        }

        bytes32[] memory contextColumn = LibBytes32Array.arrayFrom(
            bytes32(uint256(uint160(token))),
            vaultId,
            Float.unwrap(beforeBalance),
            Float.unwrap(afterBalance),
            Float.unwrap(targetAmount),
            bytes32(uint256(decimals))
        );

        bytes32[][] memory context = LibBytes32Matrix.matrixFrom(contextColumn);

        if (post.length != 0) {
            LibOrderBook.doPost(context, post);
        }
    }

    /// @inheritdoc IRaindexV6
    function addOrder4(OrderConfigV4 calldata orderConfig, TaskV2[] calldata post)
        external
        nonReentrant
        returns (bool)
    {
        if (orderConfig.validInputs.length == 0) {
            revert OrderNoInputs();
        }
        if (orderConfig.validOutputs.length == 0) {
            revert OrderNoOutputs();
        }

        // Merge our view on the sender/owner and handle IO emptiness with the
        // config and deployer's view on the `EvaluableV2` to produce the final
        // order.
        OrderV4 memory order = OrderV4(
            msg.sender, orderConfig.evaluable, orderConfig.validInputs, orderConfig.validOutputs, orderConfig.nonce
        );
        bytes32 orderHash = order.hash();

        bool stateChange = sOrders[orderHash] == ORDER_DEAD;

        // If the order is not dead we return early without state changes.
        if (stateChange) {
            // This has to come after the external call to deploy the expression
            // because the order hash is derived from the expression and DISPair
            // addresses.
            //slither-disable-next-line reentrancy-benign
            sOrders[orderHash] = ORDER_LIVE;
            emit AddOrderV3(order.owner, orderHash, order);

            // We only emit the meta event if there is meta to emit. We do require
            // that the meta self describes as a Rain meta document.
            if (orderConfig.meta.length > 0) {
                LibMeta.checkMetaUnhashedV1(orderConfig.meta);
                emit MetaV1_2(order.owner, orderHash, orderConfig.meta);
            }

            LibOrderBook.doPost(
                LibBytes32Matrix.matrixFrom(
                    LibBytes32Array.arrayFrom(orderHash, bytes32(uint256(uint160(msg.sender))))
                ),
                post
            );
        }

        return stateChange;
    }

    /// @inheritdoc IRaindexV6
    function removeOrder3(OrderV4 calldata order, TaskV2[] calldata post)
        external
        nonReentrant
        returns (bool stateChanged)
    {
        if (msg.sender != order.owner) {
            revert NotOrderOwner(order.owner);
        }
        bytes32 orderHash = order.hash();
        if (sOrders[orderHash] == ORDER_LIVE) {
            stateChanged = true;
            sOrders[orderHash] = ORDER_DEAD;
            emit RemoveOrderV3(msg.sender, orderHash, order);

            LibOrderBook.doPost(
                LibBytes32Matrix.matrixFrom(
                    LibBytes32Array.arrayFrom(orderHash, bytes32(uint256(uint160(msg.sender))))
                ),
                post
            );
        }
    }

    /// @dev Reverts with `TokenSelfTrade` if the input and output tokens are
    /// the same address.
    function checkTokenSelfTrade(OrderV4 memory order, uint256 inputIOIndex, uint256 outputIOIndex) internal pure {
        if (order.validInputs[inputIOIndex].token == order.validOutputs[outputIOIndex].token) {
            revert TokenSelfTrade();
        }
    }

    /// @inheritdoc IRaindexV6
    function quote2(QuoteV2 calldata quoteConfig) external view returns (bool, Float, Float) {
        bytes32 orderHash = quoteConfig.order.hash();

        if (sOrders[orderHash] != ORDER_LIVE) {
            return (false, Float.wrap(0), Float.wrap(0));
        }

        checkTokenSelfTrade(quoteConfig.order, quoteConfig.inputIOIndex, quoteConfig.outputIOIndex);

        OrderIOCalculationV4 memory orderIOCalculation = calculateOrderIO(
            quoteConfig.order,
            quoteConfig.inputIOIndex,
            quoteConfig.outputIOIndex,
            msg.sender,
            quoteConfig.signedContext
        );
        return (true, orderIOCalculation.outputMax, orderIOCalculation.IORatio);
    }

    /// @inheritdoc IRaindexV6
    // Most of the cyclomatic complexity here is due to the error handling within
    // the loop. The actual logic is fairly linear.
    //slither-disable-next-line cyclomatic-complexity
    function takeOrders4(TakeOrdersConfigV5 calldata config)
        external
        nonReentrant
        returns (Float totalTakerInput, Float totalTakerOutput)
    {
        if (config.orders.length == 0) {
            revert NoOrders();
        }

        TakeOrderConfigV4 memory takeOrderConfig;
        OrderV4 memory order;
        address orderInputToken = config.orders[0].order.validInputs[config.orders[0].inputIOIndex].token;
        address orderOutputToken = config.orders[0].order.validOutputs[config.orders[0].outputIOIndex].token;

        // Allocate a region of memory to hold pointers. We don't know how many
        // will run at this point, but we conservatively set aside a slot for
        // every order in case we need it, rather than attempt to dynamically
        // resize the array later. There's no guarantee that a dynamic solution
        // would even be cheaper gas-wise, and it would almost certainly be more
        // complex.
        OrderIOCalculationV4[] memory orderIOCalculationsToHandle;
        {
            uint256 length = config.orders.length;
            assembly ("memory-safe") {
                let ptr := mload(0x40)
                orderIOCalculationsToHandle := ptr
                mstore(0x40, add(ptr, mul(add(length, 1), 0x20)))
            }
        }

        {
            Float remainingTakerIO = config.maximumIO;
            if (!remainingTakerIO.gt(Float.wrap(0))) {
                revert ZeroMaximumIO();
            }

            uint256 i = 0;
            while (i < config.orders.length && remainingTakerIO.gt(Float.wrap(0))) {
                takeOrderConfig = config.orders[i];
                order = takeOrderConfig.order;
                // Every order needs the same input token.
                // Every order needs the same output token.
                if (
                    (order.validInputs[takeOrderConfig.inputIOIndex].token != orderInputToken)
                        || (order.validOutputs[takeOrderConfig.outputIOIndex].token != orderOutputToken)
                ) {
                    revert TokenMismatch();
                }

                checkTokenSelfTrade(order, takeOrderConfig.inputIOIndex, takeOrderConfig.outputIOIndex);

                bytes32 orderHash = order.hash();
                if (sOrders[orderHash] == ORDER_DEAD) {
                    emit OrderNotFound(msg.sender, order.owner, orderHash);
                } else {
                    OrderIOCalculationV4 memory orderIOCalculation = calculateOrderIO(
                        order,
                        takeOrderConfig.inputIOIndex,
                        takeOrderConfig.outputIOIndex,
                        msg.sender,
                        takeOrderConfig.signedContext
                    );

                    // Skip orders that are too expensive rather than revert as we have
                    // no way of knowing if a specific order becomes too expensive
                    // between submitting to mempool and execution, but other orders may
                    // be valid so we want to take advantage of those if possible.
                    if (orderIOCalculation.IORatio.gt(config.maximumIORatio)) {
                        emit OrderExceedsMaxRatio(msg.sender, order.owner, orderHash);
                    } else if (orderIOCalculation.outputMax.isZero()) {
                        emit OrderZeroAmount(msg.sender, order.owner, orderHash);
                    } else {
                        Float takerInput;
                        Float takerOutput;
                        if (config.IOIsInput) {
                            // Taker is just "market buying" the order output max.
                            // Can't exceed the remaining taker input.
                            takerInput = orderIOCalculation.outputMax.min(remainingTakerIO);

                            // Slither false positive because it sees the div on
                            // the else branch and triggers a
                            // divide-before-multiply warning. In this case the
                            // IOIsInput on the config is always the same for
                            // every order in the loop so we always take the same
                            // branch, whether this or the else.
                            //slither-disable-next-line divide-before-multiply
                            takerOutput = orderIOCalculation.IORatio.mul(takerInput);

                            remainingTakerIO = remainingTakerIO.sub(takerInput);
                        } else {
                            // Taker is "market selling" up to the order output max.
                            Float orderMaxInput = orderIOCalculation.IORatio.mul(orderIOCalculation.outputMax);
                            takerOutput = orderMaxInput.min(remainingTakerIO);
                            // This rounds down which favours the order/dex.
                            takerInput = takerOutput.div(orderIOCalculation.IORatio);

                            remainingTakerIO = remainingTakerIO.sub(takerOutput);
                        }

                        totalTakerOutput = totalTakerOutput.add(takerOutput);
                        totalTakerInput = totalTakerInput.add(takerInput);

                        recordVaultIO(takerOutput, takerInput, orderIOCalculation);
                        emit TakeOrderV3(msg.sender, takeOrderConfig, takerInput, takerOutput);

                        // Add the pointer to the order IO calculation to the array
                        // of order IO calculations to handle. This is
                        // unconditional because conditional behaviour is checked
                        // in `handleIO` and we don't want to duplicate that.
                        assembly ("memory-safe") {
                            // Inc the length by 1.
                            let newLength := add(mload(orderIOCalculationsToHandle), 1)
                            mstore(orderIOCalculationsToHandle, newLength)
                            // Store the pointer to the order IO calculation.
                            mstore(add(orderIOCalculationsToHandle, mul(newLength, 0x20)), orderIOCalculation)
                        }
                    }
                }

                unchecked {
                    i++;
                }
            }
        }

        {
            Float actualRelevantTakerIO = config.IOIsInput ? totalTakerInput : totalTakerOutput;
            if (actualRelevantTakerIO.lt(config.minimumIO)) {
                revert MinimumIO(config.minimumIO, actualRelevantTakerIO);
            }
        }

        // We send the tokens to `msg.sender` first adopting a similar pattern to
        // Uniswap flash swaps. We call the caller before attempting to pull
        // tokens from them in order to facilitate better integrations with
        // external liquidity sources. This could be done by the caller using
        // flash loans but this callback:
        // - may be simpler for the caller to implement
        // - allows the caller to call `takeOrders` _before_ placing external
        //   trades, which is important if the order logic itself is dependent on
        //   external data (e.g. prices) that could be modified by the caller's
        //   trades.

        pushTokens(msg.sender, orderOutputToken, totalTakerInput);

        if (config.data.length > 0) {
            IRaindexV6OrderTaker(msg.sender)
                .onTakeOrders2(orderOutputToken, orderInputToken, totalTakerInput, totalTakerOutput, config.data);
        }

        pullTokens(msg.sender, orderInputToken, totalTakerOutput);

        unchecked {
            for (uint256 i = 0; i < orderIOCalculationsToHandle.length; i++) {
                handleIO(orderIOCalculationsToHandle[i]);
            }
        }
    }

    /// @inheritdoc IRaindexV6
    function clear3(
        OrderV4 memory aliceOrder,
        OrderV4 memory bobOrder,
        ClearConfigV2 calldata clearConfig,
        SignedContextV1[] memory aliceSignedContext,
        SignedContextV1[] memory bobSignedContext
    ) external nonReentrant {
        {
            if (aliceOrder.owner == bobOrder.owner) {
                revert SameOwner();
            }
            if (
                (aliceOrder.validOutputs[clearConfig.aliceOutputIOIndex].token
                            != bobOrder.validInputs[clearConfig.bobInputIOIndex].token)
                    || (bobOrder.validOutputs[clearConfig.bobOutputIOIndex].token
                            != aliceOrder.validInputs[clearConfig.aliceInputIOIndex].token)
            ) {
                revert TokenMismatch();
            }

            if (
                aliceOrder.validInputs[clearConfig.aliceInputIOIndex].token
                    == aliceOrder.validOutputs[clearConfig.aliceOutputIOIndex].token
            ) {
                revert TokenSelfTrade();
            }

            // If either order is dead the clear is a no-op other than emitting
            // `OrderNotFound`. Returning rather than erroring makes it easier to
            // bulk clear using `Multicall`.
            if (sOrders[aliceOrder.hash()] == ORDER_DEAD) {
                emit OrderNotFound(msg.sender, aliceOrder.owner, aliceOrder.hash());
                return;
            }
            if (sOrders[bobOrder.hash()] == ORDER_DEAD) {
                emit OrderNotFound(msg.sender, bobOrder.owner, bobOrder.hash());
                return;
            }

            // Emit the Clear event before `eval2`.
            emit ClearV3(msg.sender, aliceOrder, bobOrder, clearConfig);
        }
        OrderIOCalculationV4 memory aliceOrderIOCalculation = calculateOrderIO(
            aliceOrder, clearConfig.aliceInputIOIndex, clearConfig.aliceOutputIOIndex, bobOrder.owner, bobSignedContext
        );
        OrderIOCalculationV4 memory bobOrderIOCalculation = calculateOrderIO(
            bobOrder, clearConfig.bobInputIOIndex, clearConfig.bobOutputIOIndex, aliceOrder.owner, aliceSignedContext
        );

        ClearStateChangeV2 memory clearStateChange =
            calculateClearStateChange(aliceOrderIOCalculation, bobOrderIOCalculation);

        recordVaultIO(clearStateChange.aliceInput, clearStateChange.aliceOutput, aliceOrderIOCalculation);
        recordVaultIO(clearStateChange.bobInput, clearStateChange.bobOutput, bobOrderIOCalculation);

        {
            Float aliceBounty = clearStateChange.aliceOutput.sub(clearStateChange.bobInput);
            Float bobBounty = clearStateChange.bobOutput.sub(clearStateChange.aliceInput);

            // A negative bounty means there is a spread between the orders.
            // This is a critical error because it means the DEX could be
            // exploited if allowed.
            if (aliceBounty.lt(Float.wrap(0)) || bobBounty.lt(Float.wrap(0))) {
                revert NegativeBounty();
            }

            increaseVaultBalance(
                msg.sender,
                aliceOrder.validOutputs[clearConfig.aliceOutputIOIndex].token,
                clearConfig.aliceBountyVaultId,
                aliceBounty
            );
            increaseVaultBalance(
                msg.sender,
                bobOrder.validOutputs[clearConfig.bobOutputIOIndex].token,
                clearConfig.bobBountyVaultId,
                bobBounty
            );
        }

        emit AfterClearV2(msg.sender, clearStateChange);

        handleIO(aliceOrderIOCalculation);
        handleIO(bobOrderIOCalculation);

        // Do this last so we don't swallow errors from the handle IO.
        // Slither false positive. clearStateChange is a memory variable that
        // cannot be modified by reentrancy, and clear3 is nonReentrant.
        //slither-disable-next-line reentrancy-balance
        if (clearStateChange.aliceOutput.isZero() && clearStateChange.bobOutput.isZero()) {
            revert ClearZeroAmount();
        }
    }

    /// Main entrypoint into an order calculates the amount and IO ratio. Both
    /// are always treated as Float values and then rescaled according to the
    /// order's definition of each token's actual fixed point decimals.
    /// @param order The order to evaluate.
    /// @param inputIOIndex The index of the input token being calculated for.
    /// @param outputIOIndex The index of the output token being calculated for.
    /// @param counterparty The counterparty of the order as it is currently
    /// being cleared against.
    /// @param signedContext Any signed context provided by the clearer/taker
    /// that the order may need for its calculations.
    function calculateOrderIO(
        OrderV4 memory order,
        uint256 inputIOIndex,
        uint256 outputIOIndex,
        address counterparty,
        SignedContextV1[] memory signedContext
    ) internal view returns (OrderIOCalculationV4 memory) {
        unchecked {
            bytes32 orderHash = order.hash();

            bytes32[][] memory context;
            {
                bytes32[][] memory callingContext = new bytes32[][](CALLING_CONTEXT_COLUMNS);
                callingContext[CONTEXT_CALLING_CONTEXT_COLUMN - 1] = LibBytes32Array.arrayFrom(
                    orderHash, bytes32(uint256(uint160(order.owner))), bytes32(uint256(uint160(counterparty)))
                );

                {
                    (TOFUOutcome inputOutcome, uint8 inputDecimals) =
                        LibTOFUTokenDecimals.decimalsForTokenReadOnly(order.validInputs[inputIOIndex].token);
                    if (inputOutcome != TOFUOutcome.Consistent && inputOutcome != TOFUOutcome.Initial) {
                        revert ITOFUTokenDecimals.TokenDecimalsReadFailure(
                            order.validInputs[inputIOIndex].token, inputOutcome
                        );
                    }

                    Float inputTokenVaultBalance = _vaultBalance(
                        order.owner, order.validInputs[inputIOIndex].token, order.validInputs[inputIOIndex].vaultId
                    );
                    callingContext[CONTEXT_VAULT_INPUTS_COLUMN - 1] = LibBytes32Array.arrayFrom(
                        bytes32(uint256(uint160(order.validInputs[inputIOIndex].token))),
                        bytes32(uint256(inputDecimals)),
                        order.validInputs[inputIOIndex].vaultId,
                        Float.unwrap(inputTokenVaultBalance),
                        // Don't know the balance diff yet!
                        0
                    );
                }

                {
                    (TOFUOutcome outputOutcome, uint8 outputDecimals) =
                        LibTOFUTokenDecimals.decimalsForTokenReadOnly(order.validOutputs[outputIOIndex].token);
                    if (outputOutcome != TOFUOutcome.Consistent && outputOutcome != TOFUOutcome.Initial) {
                        revert ITOFUTokenDecimals.TokenDecimalsReadFailure(
                            order.validOutputs[outputIOIndex].token, outputOutcome
                        );
                    }

                    Float outputTokenVaultBalance = _vaultBalance(
                        order.owner, order.validOutputs[outputIOIndex].token, order.validOutputs[outputIOIndex].vaultId
                    );
                    callingContext[CONTEXT_VAULT_OUTPUTS_COLUMN - 1] = LibBytes32Array.arrayFrom(
                        bytes32(uint256(uint160(order.validOutputs[outputIOIndex].token))),
                        bytes32(uint256(outputDecimals)),
                        order.validOutputs[outputIOIndex].vaultId,
                        Float.unwrap(outputTokenVaultBalance),
                        // Don't know the balance diff yet!
                        0
                    );
                }

                context = LibContext.build(callingContext, signedContext);
            }

            // The state changes produced here are handled in _recordVaultIO so
            // that local storage writes happen before writes on the interpreter.
            StateNamespace namespace = StateNamespace.wrap(uint256(uint160(order.owner)));
            // Slither false positive. External calls within loops are fine if
            // the caller controls which orders are eval'd as they can drop
            // failing calls and resubmit a new transaction.
            // https://github.com/crytic/slither/issues/880
            //slither-disable-next-line calls-loop
            (StackItem[] memory calculateOrderStack, bytes32[] memory calculateOrderKVs) = order.evaluable.interpreter
                .eval4(
                    EvalV4({
                        store: order.evaluable.store,
                        namespace: LibNamespace.qualifyNamespace(namespace, address(this)),
                        bytecode: order.evaluable.bytecode,
                        sourceIndex: CALCULATE_ORDER_ENTRYPOINT,
                        context: context,
                        inputs: new StackItem[](0),
                        stateOverlay: new bytes32[](0)
                    })
                );

            // This is a much clearer error message and overall is more efficient
            // than solidity generic index out of bounds errors.
            if (calculateOrderStack.length < CALCULATE_ORDER_MIN_OUTPUTS) {
                revert UnsupportedCalculateOutputs(calculateOrderStack.length);
            }

            Float orderIORatio;
            Float orderOutputMax;
            assembly ("memory-safe") {
                orderIORatio := mload(add(calculateOrderStack, 0x20))
                orderOutputMax := mload(add(calculateOrderStack, 0x40))
            }

            {
                // The order owner can't send more than the smaller of their vault
                // balance or their per-order limit.
                IOV2 memory output = order.validOutputs[outputIOIndex];
                Float ownerVaultBalance = _vaultBalance(order.owner, output.token, output.vaultId);
                orderOutputMax = orderOutputMax.min(ownerVaultBalance);
            }

            // Populate the context with the output max rescaled and vault capped.
            context[CONTEXT_CALCULATIONS_COLUMN] =
                LibBytes32Array.arrayFrom(Float.unwrap(orderOutputMax), Float.unwrap(orderIORatio));

            return OrderIOCalculationV4({
                order: order,
                outputIOIndex: outputIOIndex,
                outputMax: orderOutputMax,
                IORatio: orderIORatio,
                context: context,
                namespace: namespace,
                kvs: calculateOrderKVs
            });
        }
    }

    /// @dev Increases a vault balance by `amount`. For vault ID 0, pushes tokens
    /// directly to the owner instead. Returns (oldBalance, newBalance).
    function increaseVaultBalance(address owner, address token, bytes32 vaultId, Float amount)
        internal
        returns (Float, Float)
    {
        if (amount.lt(Float.wrap(0))) {
            revert NegativeVaultBalanceChange(amount);
        }

        // Vault ID 0 is special, it directly transfers tokens to the owner
        // rather than allocating an internal balance in a vault.
        //slither-disable-next-line incorrect-equality
        if (vaultId == bytes32(0)) {
            pushTokens(owner, token, amount);
            // The internal balance of vault 0 is always 0 as every transfer in
            // is effectively a compound matching withdrawal.
            return (Float.wrap(0), Float.wrap(0));
        } else {
            Float oldBalance = sVaultBalances[owner][token][vaultId];
            Float newBalance = oldBalance.add(amount);

            // This should never be possible as amount is positive and floats are
            // effectively impossible to overflow, but we check it anyway to be safe.
            if (newBalance.lt(Float.wrap(0))) {
                revert NegativeVaultBalance(newBalance);
            }
            sVaultBalances[owner][token][vaultId] = newBalance;

            return (oldBalance, newBalance);
        }
    }

    /// @dev Decreases a vault balance by `amount`. For vault ID 0, pulls tokens
    /// directly from the owner instead. Returns (oldBalance, newBalance).
    function decreaseVaultBalance(address owner, address token, bytes32 vaultId, Float amount)
        internal
        returns (Float, Float)
    {
        if (amount.lt(Float.wrap(0))) {
            revert NegativeVaultBalanceChange(amount);
        }

        // Vault ID 0 is special, it directly pulls tokens from the owner when
        // their balance on DEX is reduced.
        //slither-disable-next-line incorrect-equality
        if (vaultId == bytes32(0)) {
            pullTokens(owner, token, amount);
            // The internal balance of vault 0 is always 0 as every balance
            // decrease is effectively a compound matching deposit.
            return (Float.wrap(0), Float.wrap(0));
        } else {
            Float oldBalance = sVaultBalances[owner][token][vaultId];
            Float newBalance = oldBalance.sub(amount);

            // This can definitely happen, so needs to be guarded against.
            // There's no specific check anywhere else that vault balances don't go
            // negative, so this function should be used everywhere for safety.
            if (newBalance.lt(Float.wrap(0))) {
                revert NegativeVaultBalance(newBalance);
            }
            sVaultBalances[owner][token][vaultId] = newBalance;

            return (oldBalance, newBalance);
        }
    }

    /// @dev Given an order, final input and output amounts and the IO
    /// calculation verbatim from `_calculateOrderIO`, update the order owner's
    /// vault balances accordingly. The input and output balance diffs are
    /// written into the context matrix before vault balances are adjusted.
    /// @param input The Float input amount to credit to the order owner's
    /// input vault.
    /// @param output The Float output amount to debit from the order owner's
    /// output vault.
    /// @param orderIOCalculation The order IO calculation produced by
    /// `_calculateOrderIO`, containing the order, context matrix and
    /// namespace.
    function recordVaultIO(Float input, Float output, OrderIOCalculationV4 memory orderIOCalculation) internal {
        orderIOCalculation.context[CONTEXT_VAULT_INPUTS_COLUMN][CONTEXT_VAULT_IO_BALANCE_DIFF] = Float.unwrap(input);
        orderIOCalculation.context[CONTEXT_VAULT_OUTPUTS_COLUMN][CONTEXT_VAULT_IO_BALANCE_DIFF] = Float.unwrap(output);

        increaseVaultBalance(
            orderIOCalculation.order.owner,
            address(uint160(uint256(orderIOCalculation.context[CONTEXT_VAULT_INPUTS_COLUMN][CONTEXT_VAULT_IO_TOKEN]))),
            orderIOCalculation.context[CONTEXT_VAULT_INPUTS_COLUMN][CONTEXT_VAULT_IO_VAULT_ID],
            input
        );
        // Increase before decreasing so that if vault id == 0 then we push
        // tokens before pulling them.
        decreaseVaultBalance(
            orderIOCalculation.order.owner,
            address(uint160(uint256(orderIOCalculation.context[CONTEXT_VAULT_OUTPUTS_COLUMN][CONTEXT_VAULT_IO_TOKEN]))),
            orderIOCalculation.context[CONTEXT_VAULT_OUTPUTS_COLUMN][CONTEXT_VAULT_IO_VAULT_ID],
            output
        );

        // Emit the context only once in its fully populated form rather than two
        // nearly identical emissions of a partial and full context.
        emit ContextV2(msg.sender, orderIOCalculation.context);
    }

    /// @dev Persists interpreter state writes then evaluates the handle IO
    /// entrypoint for an order, if it has one.
    function handleIO(OrderIOCalculationV4 memory orderIOCalculation) internal {
        // Apply state changes to the interpreter store after the vault balances
        // are updated, but before we call handle IO. We want handle IO to see
        // a consistent view on sets from calculate IO.
        if (orderIOCalculation.kvs.length > 0) {
            // Slither false positive. External calls within loops are fine if
            // the caller controls which orders are eval'd as they can drop
            // failing calls and resubmit a new transaction.
            // https://github.com/crytic/slither/issues/880
            //slither-disable-next-line calls-loop
            orderIOCalculation.order.evaluable.store.set(orderIOCalculation.namespace, orderIOCalculation.kvs);
        }

        // The handle IO eval is run under the same namespace as the
        // calculate order entrypoint.
        // Slither false positive. External calls within loops are fine if
        // the caller controls which orders are eval'd as they can drop
        // failing calls and resubmit a new transaction.
        // https://github.com/crytic/slither/issues/880
        //slither-disable-next-line calls-loop,unused-return
        (, bytes32[] memory handleIOKVs) = orderIOCalculation.order.evaluable.interpreter
            .eval4(
                EvalV4({
                    store: orderIOCalculation.order.evaluable.store,
                    namespace: LibNamespace.qualifyNamespace(orderIOCalculation.namespace, address(this)),
                    bytecode: orderIOCalculation.order.evaluable.bytecode,
                    sourceIndex: HANDLE_IO_ENTRYPOINT,
                    context: orderIOCalculation.context,
                    inputs: new StackItem[](0),
                    stateOverlay: new bytes32[](0)
                })
            );
        // Apply state changes to the interpreter store from the handle IO
        // entrypoint.
        if (handleIOKVs.length > 0) {
            // Slither false positive. External calls within loops are fine
            // if the caller controls which orders are eval'd as they can
            // drop failing calls and resubmit a new transaction.
            // https://github.com/crytic/slither/issues/880
            //slither-disable-next-line calls-loop
            orderIOCalculation.order.evaluable.store.set(orderIOCalculation.namespace, handleIOKVs);
        }
    }

    /// Calculates the clear state change given both order calculations for order
    /// alice and order bob. The input of each is their output multiplied by
    /// their IO ratio and the output of each is the smaller of their maximum
    /// output and the counterparty IO * max output.
    /// @param aliceOrderIOCalculation Order calculation for Alice.
    /// @param bobOrderIOCalculation Order calculation for Bob.
    /// @return clearStateChange The clear state change with absolute inputs and
    /// outputs for Alice and Bob.
    function calculateClearStateChange(
        OrderIOCalculationV4 memory aliceOrderIOCalculation,
        OrderIOCalculationV4 memory bobOrderIOCalculation
    ) internal pure returns (ClearStateChangeV2 memory clearStateChange) {
        // Calculate the clear state change for Alice.
        (clearStateChange.aliceInput, clearStateChange.aliceOutput) =
            calculateClearStateAlice(aliceOrderIOCalculation, bobOrderIOCalculation);

        // Flip alice and bob to calculate bob's output.
        (clearStateChange.bobInput, clearStateChange.bobOutput) =
            calculateClearStateAlice(bobOrderIOCalculation, aliceOrderIOCalculation);
    }

    /// @dev Calculates Alice's input and output given both order calculations.
    /// Alice's output is capped by Bob's max output, and her input is derived
    /// from her IO ratio.
    function calculateClearStateAlice(
        OrderIOCalculationV4 memory aliceOrderIOCalculation,
        OrderIOCalculationV4 memory bobOrderIOCalculation
    ) internal pure returns (Float aliceInput, Float aliceOutput) {
        // Alice's input is her output * her IO ratio.
        aliceInput = aliceOrderIOCalculation.outputMax.mul(aliceOrderIOCalculation.IORatio);

        aliceOutput = aliceOrderIOCalculation.outputMax;

        // If Alice's input is greater than Bob's max output, Alice's input is
        // capped at Bob's max output.
        if (aliceInput.gt(bobOrderIOCalculation.outputMax)) {
            aliceInput = bobOrderIOCalculation.outputMax;

            // Alice's output is capped at her input / her IO ratio.
            aliceOutput = aliceInput.div(aliceOrderIOCalculation.IORatio);
        }
    }

    /// @dev Pulls `amount` of `token` from `account` via `safeTransferFrom`.
    /// Returns the fixed-decimal amount transferred and the token decimals.
    function pullTokens(address account, address token, Float amount) internal returns (uint256, uint8) {
        (TOFUOutcome tofuOutcome, uint8 decimals) = LibTOFUTokenDecimals.decimalsForToken(token);
        if (tofuOutcome != TOFUOutcome.Consistent && tofuOutcome != TOFUOutcome.Initial) {
            revert ITOFUTokenDecimals.TokenDecimalsReadFailure(token, tofuOutcome);
        }
        if (amount.lt(Float.wrap(0))) {
            revert NegativePull();
        }

        (uint256 amount18, bool lossless) = LibDecimalFloat.toFixedDecimalLossy(amount, decimals);
        // Round truncation up when pulling.
        if (!lossless) {
            // This needs to be checked math as an overflow would cause tokens
            // to silently not be pulled (wraps to 0).
            ++amount18;
        }
        if (amount18 > 0) {
            IERC20(token).safeTransferFrom(account, address(this), amount18);
        }
        return (amount18, decimals);
    }

    /// @dev Pushes `amountFloat` of `token` to `account` via `safeTransfer`.
    /// Returns the fixed-decimal amount transferred and the token decimals.
    function pushTokens(address account, address token, Float amountFloat) internal returns (uint256, uint8) {
        (TOFUOutcome tofuOutcome, uint8 decimals) = LibTOFUTokenDecimals.decimalsForToken(token);
        if (tofuOutcome != TOFUOutcome.Consistent && tofuOutcome != TOFUOutcome.Initial) {
            revert ITOFUTokenDecimals.TokenDecimalsReadFailure(token, tofuOutcome);
        }

        if (amountFloat.lt(Float.wrap(0))) {
            revert NegativePush();
        }

        //slither-disable-next-line unused-return
        (uint256 amount,) = LibDecimalFloat.toFixedDecimalLossy(amountFloat, decimals);
        if (amount > 0) {
            IERC20(token).safeTransfer(account, amount);
        }

        return (amount, decimals);
    }

    /// @dev Reverts with `ZeroVaultId` if `vaultId` is zero.
    function _nonZeroVaultId(address vaultOwner, address token, bytes32 vaultId) internal pure {
        if (vaultId == bytes32(0)) {
            revert ZeroVaultId(vaultOwner, token);
        }
    }

    modifier nonZeroVaultId(address vaultOwner, address token, bytes32 vaultId) {
        _nonZeroVaultId(vaultOwner, token, vaultId);
        _;
    }
}
