// SPDX-License-Identifier: LicenseRef-DCL-1.0
// SPDX-FileCopyrightText: Copyright (c) 2020 Rain Open Source Software Ltd
pragma solidity =0.8.25;

import {Vm} from "forge-std/Vm.sol";
import {OrderBookV6ExternalRealTest, IERC20} from "test/util/abstract/OrderBookV6ExternalRealTest.sol";
import {
    OrderV4,
    TakeOrderConfigV4,
    IOV2,
    OrderConfigV4,
    TakeOrdersConfigV5,
    EvaluableV4,
    TaskV2
} from "rain.raindex.interface/interface/IRaindexV6.sol";
import {SourceIndexOutOfBounds} from "rain.interpreter.interface/error/ErrBytecode.sol";
import {Float, LibDecimalFloat} from "rain.math.float/lib/LibDecimalFloat.sol";
import {IERC20Metadata} from "openzeppelin-contracts/contracts/token/ERC20/extensions/IERC20Metadata.sol";
import {LibTestTakeOrder} from "test/util/lib/LibTestTakeOrder.sol";

/// @title OrderBookV6TakeOrderHandleIORevertTest
/// @notice A test harness for testing the OrderBook takeOrder function will run
/// handle IO and revert if it fails.
contract OrderBookV6TakeOrderHandleIORevertTest is OrderBookV6ExternalRealTest {
    using LibDecimalFloat for Float;

    function checkTakeOrderHandleIO(bytes[] memory configs, bytes memory err, Float maxInput) internal {
        checkTakeOrderHandleIO(configs, err, maxInput, bytes32(uint256(0x01)), bytes32(uint256(0x01)));
    }

    function checkTakeOrderHandleIO(
        bytes[] memory configs,
        bytes memory err,
        Float maxInput,
        bytes32 outputVaultId,
        bytes32 inputVaultId
    ) internal {
        bytes32 vaultId = outputVaultId;
        address inputToken = address(0x100);
        address outputToken = address(0x101);

        OrderConfigV4 memory config;
        IOV2[] memory validOutputs;
        IOV2[] memory validInputs;
        {
            validInputs = new IOV2[](1);
            validInputs[0] = IOV2(inputToken, inputVaultId);
            validOutputs = new IOV2[](1);
            validOutputs[0] = IOV2(outputToken, outputVaultId);
            // Etch with invalid.
            vm.etch(outputToken, hex"fe");
            vm.etch(inputToken, hex"fe");
            // Mock every call to output as a success, so the orderbook thinks it
            // is transferring tokens.
            vm.mockCall(outputToken, abi.encodeWithSelector(IERC20Metadata.decimals.selector), abi.encode(uint8(18)));
            vm.mockCall(
                outputToken,
                abi.encodeWithSelector(IERC20.transferFrom.selector, address(this), address(iOrderbook)),
                abi.encode(true)
            );
            vm.mockCall(outputToken, abi.encodeWithSelector(IERC20.transfer.selector, address(this)), abi.encode(true));
            vm.mockCall(inputToken, bytes(""), abi.encode(true));
        }
        if (outputVaultId == bytes32(0)) {
            mockVault0Output(outputToken, address(this), uint256(int256(type(int224).max)));
        } else {
            iOrderbook.deposit4(
                outputToken, vaultId, LibDecimalFloat.packLossless(type(int224).max, -18), new TaskV2[](0)
            );
            assertTrue(
                iOrderbook.vaultBalance2(address(this), outputToken, vaultId)
                    .eq(LibDecimalFloat.packLossless(type(int224).max, -18))
            );
        }
        if (inputVaultId == bytes32(0)) {
            mockVault0Input(inputToken, address(this), 0);
        }

        TakeOrderConfigV4[] memory orders = new TakeOrderConfigV4[](configs.length);

        for (uint256 i = 0; i < configs.length; i++) {
            bytes memory bytecode = iParserV2.parse2(configs[i]);
            EvaluableV4 memory evaluable = EvaluableV4(iInterpreter, iStore, bytecode);
            config = OrderConfigV4(evaluable, validInputs, validOutputs, bytes32(i), bytes32(0), "");

            vm.recordLogs();
            iOrderbook.addOrder4(config, new TaskV2[](0));
            Vm.Log[] memory entries = vm.getRecordedLogs();
            assertEq(entries.length, 1);
            OrderV4 memory order = LibTestTakeOrder.extractOrderFromLogs(entries);

            orders[i] = LibTestTakeOrder.wrapSingle(order)[0];
        }
        TakeOrdersConfigV5 memory takeOrdersConfig = LibTestTakeOrder.defaultTakeConfig(orders);
        takeOrdersConfig.maximumIO = maxInput;

        if (err.length > 0) {
            vm.expectRevert(err);
        }
        (Float totalTakerInput, Float totalTakerOutput) = iOrderbook.takeOrders4(takeOrdersConfig);
        // We don't really care about the outputs as the tests are basically just
        // trying to show that the IO handler is running or not running by simple
        // reverts.
        (totalTakerInput, totalTakerOutput);
    }

    function testTakeOrderHandleIO00() external {
        bytes[] memory configs = new bytes[](1);
        configs[0] = "_ _:max-positive-value() 1;:ensure(0 \"err\");";
        checkTakeOrderHandleIO(configs, "err", LibDecimalFloat.packLossless(type(int224).max, 0));
    }

    function testTakeOrderHandleIO1() external {
        bytes[] memory configs = new bytes[](2);
        configs[0] = "_ _:1 1;:ensure(0 \"err\");";
        configs[1] = "_ _:1 1;:;";
        checkTakeOrderHandleIO(configs, "err", LibDecimalFloat.packLossless(type(int224).max, 0));
    }

    function testTakeOrderHandleIO2() external {
        bytes[] memory configs = new bytes[](2);
        configs[0] = "_ _:1 1;:;";
        configs[1] = "_ _:1 1;:ensure(0 \"err\");";
        checkTakeOrderHandleIO(configs, "err", LibDecimalFloat.packLossless(type(int224).max, 0));
    }

    function testTakeOrderHandleIO3() external {
        bytes[] memory configs = new bytes[](3);
        configs[0] = "_ _:1 1;:;";
        configs[1] = "_ _:1 1;:ensure(0 \"err\");";
        configs[2] = "_ _:1 1;:;";
        checkTakeOrderHandleIO(configs, "err", LibDecimalFloat.packLossless(type(int224).max, 0));
    }

    function testTakeOrderHandleIO4() external {
        bytes[] memory configs = new bytes[](3);
        configs[0] = "_ _:1 1;:;";
        configs[1] = "_ _:1 1;:ensure(0 \"err 1\");";
        configs[2] = "_ _:1 1;:ensure(0 \"err 2\");";
        checkTakeOrderHandleIO(configs, "err 1", LibDecimalFloat.packLossless(type(int224).max, 0));
    }

    function testTakeOrderHandleIO5() external {
        bytes[] memory configs = new bytes[](3);
        configs[0] = "_ _:1 1;:;";
        configs[1] = "_ _:1 1;:ensure(0 \"err 2\");";
        configs[2] = "_ _:1 1;:ensure(0 \"err 1\");";
        checkTakeOrderHandleIO(configs, "err 2", LibDecimalFloat.packLossless(type(int224).max, 0));
    }

    function testTakeOrderHandleIO6() external {
        bytes[] memory configs = new bytes[](3);
        configs[0] = "_ _:1 1;:ensure(0 \"err 2\");";
        configs[1] = "_ _:1 1;:;";
        configs[2] = "_ _:1 1;:ensure(0 \"err 1\");";
        checkTakeOrderHandleIO(configs, "err 2", LibDecimalFloat.packLossless(type(int224).max, 0));
    }

    /// forge-config: default.fuzz.runs = 100
    function testTakeOrderHandleIO7(uint256 toClear18) external {
        toClear18 = bound(toClear18, 3e18 + 1, uint256(int256(type(int224).max)));
        Float toClear = LibDecimalFloat.fromFixedDecimalLosslessPacked(toClear18, 18);
        bytes[] memory configs = new bytes[](4);
        configs[0] = "_ _:1 1;:set(0 1);";
        configs[1] = "_ _:1 1;:ensure(get(0) \"err 1\");";
        configs[2] = "_ _:1 1;:set(0 0);";
        configs[3] = "_ _:1 1;:ensure(get(0) \"err 2\");";
        checkTakeOrderHandleIO(configs, "err 2", toClear);
    }

    /// forge-config: default.fuzz.runs = 100
    function testTakeOrderHandleIO8(uint256 toClear18) external {
        toClear18 = bound(toClear18, 4e18 + 1, uint256(int256(type(int224).max)));
        Float toClear = LibDecimalFloat.fromFixedDecimalLosslessPacked(toClear18, 18);
        bytes[] memory configs = new bytes[](5);
        configs[0] = "_ _:1 1;:;";
        configs[1] = "_ _:1 1;:set(0 1);";
        configs[2] = "_ _:1 1;:ensure(get(0) \"err 1\");";
        configs[3] = "_ _:1 1;:set(0 0);";
        configs[4] = "_ _:1 1;:ensure(get(0) \"err 2\");";
        checkTakeOrderHandleIO(configs, "err 2", toClear);
    }

    // This one WONT error because the take orders stops executing the handle IO
    // before it clears 4e18 + 1, so it never hits the second ensure condition.
    /// forge-config: default.fuzz.runs = 100
    function testTakeOrderHandleIO9(uint256 toClear18) external {
        toClear18 = bound(toClear18, 1, 4e18);
        Float toClear = LibDecimalFloat.fromFixedDecimalLosslessPacked(toClear18, 18);
        bytes[] memory configs = new bytes[](5);
        configs[0] = "_ _:1 1;:;";
        configs[1] = "_ _:1 1;:set(0 1);";
        configs[2] = "_ _:1 1;:ensure(get(0) \"err 1\");";
        configs[3] = "_ _:1 1;:set(0 0);";
        configs[4] = "_ _:1 1;:ensure(get(0) \"err 2\");";
        checkTakeOrderHandleIO(configs, "", toClear);
    }

    // This one WONT error because the take orders stops executing the handle IO
    // before it clears 4e18 + 1, so it never hits the second ensure condition.
    /// forge-config: default.fuzz.runs = 100
    function testTakeOrderHandleIO10(uint256 toClear18) external {
        toClear18 = bound(toClear18, 1, 3e18);
        Float toClear = LibDecimalFloat.fromFixedDecimalLosslessPacked(toClear18, 18);
        bytes[] memory configs = new bytes[](4);
        configs[0] = "_ _:1 1;:set(0 1);";
        configs[1] = "_ _:1 1;:ensure(get(0) \"err 1\");";
        configs[2] = "_ _:1 1;:set(0 0);";
        configs[3] = "_ _:1 1;:ensure(get(0) \"err 2\");";
        checkTakeOrderHandleIO(configs, "", toClear);
    }

    /// Note that a different interpreter MAY NOT revert if handle io is missing,
    /// but the canonical interpreter will.
    function testTakeOrderNoHandleIORevert0() external {
        bytes[] memory configs = new bytes[](1);
        configs[0] = "_ _:1 1;";
        checkTakeOrderHandleIO(
            configs,
            abi.encodeWithSelector(SourceIndexOutOfBounds.selector, 1, hex"010000020200020110000001100000"),
            LibDecimalFloat.packLossless(type(int224).max, 0)
        );
    }

    /// Note that a different interpreter MAY NOT revert if handle io is missing,
    /// but the canonical interpreter will.
    function testTakeOrderNoHandleIORevert1() external {
        bytes[] memory configs = new bytes[](2);
        configs[0] = "_ _:1 1;:;";
        configs[1] = "_ _:1 1;";
        checkTakeOrderHandleIO(
            configs,
            abi.encodeWithSelector(SourceIndexOutOfBounds.selector, 1, hex"010000020200020110000001100000"),
            LibDecimalFloat.packLossless(type(int224).max, 0)
        );
    }

    /// Note that a different interpreter MAY NOT revert if handle io is missing,
    /// but the canonical interpreter will.
    function testTakeOrderNoHandleIORevert2() external {
        bytes[] memory configs = new bytes[](2);
        configs[0] = "_ _:1 1;";
        configs[1] = "_ _:1 1;:;";
        checkTakeOrderHandleIO(
            configs,
            abi.encodeWithSelector(SourceIndexOutOfBounds.selector, 1, hex"010000020200020110000001100000"),
            LibDecimalFloat.packLossless(type(int224).max, 0)
        );
    }

    function testTakeOrderHandleIO00BothVaultIdZero() external {
        bytes[] memory configs = new bytes[](1);
        configs[0] = "_ _:max-positive-value() 1;:ensure(0 \"err\");";
        checkTakeOrderHandleIO(
            configs, "err", LibDecimalFloat.packLossless(type(int224).max, 0), bytes32(0), bytes32(0)
        );
    }
}
