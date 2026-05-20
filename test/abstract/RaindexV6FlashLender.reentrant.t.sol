// SPDX-License-Identifier: LicenseRef-DCL-1.0
// SPDX-FileCopyrightText: Copyright (c) 2020 Rain Open Source Software Ltd
pragma solidity =0.8.25;

import {RaindexV6ExternalRealTest} from "test/util/abstract/RaindexV6ExternalRealTest.sol";
import {Reenteroor} from "test/util/concrete/Reenteroor.sol";
import {
    IRaindexV6,
    OrderConfigV4,
    OrderV4,
    TakeOrdersConfigV5,
    ClearConfigV2,
    TaskV2
} from "rain.raindex.interface/interface/IRaindexV6.sol";
import {IERC3156FlashBorrower} from "rain.raindex.interface/interface/ierc3156/IERC3156FlashBorrower.sol";
import {IERC20} from "openzeppelin-contracts/contracts/token/ERC20/IERC20.sol";
import {LibTestAddOrder} from "test/util/lib/LibTestAddOrder.sol";
import {LibTestTakeOrder} from "test/util/lib/LibTestTakeOrder.sol";
import {EvaluableV4, SignedContextV1} from "rain.interpreter.interface/interface/IInterpreterCallerV4.sol";
import {Float, LibDecimalFloat} from "rain.math.float/lib/LibDecimalFloat.sol";

/// @title RaindexV6FlashLenderReentrant
/// Test that flash borrowers can reenter the raindex, which is necessary for
/// trading etc. against it while the loan is active.
contract RaindexV6FlashLenderReentrant is RaindexV6ExternalRealTest {
    function checkFlashLoanNotRevert(Reenteroor borrower, bytes memory encodedCall, uint256 loanAmount) internal {
        borrower.reenterWith(encodedCall);
        vm.mockCall(
            address(iToken0),
            abi.encodeWithSelector(IERC20.transfer.selector, address(borrower), loanAmount),
            abi.encode(true)
        );
        vm.mockCall(
            address(iToken0),
            abi.encodeWithSelector(IERC20.approve.selector, address(iRaindex), loanAmount),
            abi.encode(true)
        );
        vm.mockCall(
            address(iToken0),
            abi.encodeWithSelector(IERC20.transferFrom.selector, borrower, address(iRaindex), loanAmount),
            abi.encode(true)
        );

        // Create a flash loan.
        iRaindex.flashLoan(IERC3156FlashBorrower(address(borrower)), address(iToken0), loanAmount, "");
    }

    /// Can reenter and read vault balances from within a flash loan.
    /// forge-config: default.fuzz.runs = 100
    function testReenterReadVaultBalances(uint256 vaultId, uint256 loanAmount) external {
        vm.assume(vaultId != 0);
        // Create a flash borrower.
        Reenteroor borrower = new Reenteroor();
        checkFlashLoanNotRevert(
            borrower,
            abi.encodeWithSelector(IRaindexV6.vaultBalance2.selector, address(borrower), address(iToken0), vaultId),
            loanAmount
        );
    }

    /// Can reenter and check if an order exists from within a flash loan.
    /// forge-config: default.fuzz.runs = 100
    function testReenterCheckOrderExists(bytes32 orderHash, uint256 loanAmount) external {
        // Create a flash borrower.
        Reenteroor borrower = new Reenteroor();
        checkFlashLoanNotRevert(
            borrower, abi.encodeWithSelector(IRaindexV6.orderExists.selector, orderHash), loanAmount
        );
    }

    /// Can reenter and deposit from within a flash loan.
    /// forge-config: default.fuzz.runs = 100
    function testReenterDeposit(uint256 vaultId, uint256 loanAmount, uint256 depositAmount18) external {
        vm.assume(vaultId != 0);
        depositAmount18 = bound(depositAmount18, 1, uint256(int256(type(int224).max)));
        // depositAmount18 is bound above so safe to typecast.
        // forge-lint: disable-next-line(unsafe-typecast)
        Float depositAmount = LibDecimalFloat.packLossless(int256(depositAmount18), -18);
        // Create a flash borrower.
        Reenteroor borrower = new Reenteroor();
        vm.mockCall(
            address(iToken0),
            abi.encodeWithSelector(IERC20.transferFrom.selector, borrower, address(iRaindex), depositAmount18),
            abi.encode(true)
        );
        checkFlashLoanNotRevert(
            borrower,
            abi.encodeWithSelector(
                IRaindexV6.deposit4.selector, address(iToken0), vaultId, depositAmount, new EvaluableV4[](0)
            ),
            loanAmount
        );
    }

    /// Can reenter and withdraw from within a flash loan.
    /// forge-config: default.fuzz.runs = 100
    function testReenterWithdraw(uint256 vaultId, uint256 loanAmount, uint256 withdrawAmount18) external {
        vm.assume(vaultId != 0);
        withdrawAmount18 = bound(withdrawAmount18, 1, uint256(int256(type(int224).max)));
        // withdrawAmount18 is bound above so safe to typecast.
        // forge-lint: disable-next-line(unsafe-typecast)
        Float withdrawAmount = LibDecimalFloat.packLossless(int256(withdrawAmount18), 0);
        // Create a flash borrower.
        Reenteroor borrower = new Reenteroor();
        vm.mockCall(
            address(iToken0),
            abi.encodeWithSelector(IERC20.transfer.selector, address(borrower), withdrawAmount18),
            abi.encode(true)
        );
        checkFlashLoanNotRevert(
            borrower,
            abi.encodeWithSelector(
                IRaindexV6.withdraw4.selector, address(iToken0), vaultId, withdrawAmount, new TaskV2[](0)
            ),
            loanAmount
        );
    }

    /// Can reenter and add an order from within a flash loan.
    /// forge-config: default.fuzz.runs = 100
    function testReenterAddOrder(uint256 loanAmount, OrderConfigV4 memory config) external {
        LibTestAddOrder.conformConfig(config, iInterpreter, iStore);
        config.evaluable.bytecode = iParserV2.parse2("_ _:max-positive-value() 1;:;");

        // Create a flash borrower.
        Reenteroor borrower = new Reenteroor();
        checkFlashLoanNotRevert(
            borrower, abi.encodeWithSelector(IRaindexV6.addOrder4.selector, config, new TaskV2[](0)), loanAmount
        );
    }

    /// Can reenter and remove an order from within a flash loan.
    /// forge-config: default.fuzz.runs = 100
    function testReenterRemoveOrder(uint256 loanAmount, OrderV4 memory order) external {
        // Create a flash borrower.
        Reenteroor borrower = new Reenteroor();
        order.owner = address(borrower);
        checkFlashLoanNotRevert(
            borrower, abi.encodeWithSelector(IRaindexV6.removeOrder3.selector, order, new TaskV2[](0)), loanAmount
        );
    }

    /// Can reenter and take orders.
    /// forge-config: default.fuzz.runs = 100
    function testReenterTakeOrder(uint256 loanAmount, OrderConfigV4 memory config) external {
        LibTestAddOrder.conformConfig(config, iInterpreter, iStore);
        config.validInputs[0].token = address(iToken0);
        config.validOutputs[0].token = address(iToken1);
        config.evaluable.bytecode = iParserV2.parse2("_ _:max-positive-value() 1;:;");

        vm.recordLogs();
        iRaindex.addOrder4(config, new TaskV2[](0));
        OrderV4 memory order = LibTestTakeOrder.extractOrderFromLogs(vm.getRecordedLogs());

        TakeOrdersConfigV5 memory takeOrdersConfig =
            LibTestTakeOrder.defaultTakeConfig(LibTestTakeOrder.wrapSingle(order));

        // Create a flash borrower.
        Reenteroor borrower = new Reenteroor();
        checkFlashLoanNotRevert(
            borrower, abi.encodeWithSelector(IRaindexV6.takeOrders4.selector, takeOrdersConfig), loanAmount
        );
    }

    /// Can reenter and clear orders.
    /// forge-config: default.fuzz.runs = 100
    function testReenterClear(
        uint256 loanAmount,
        address alice,
        address bob,
        OrderConfigV4 memory aliceConfig,
        OrderConfigV4 memory bobConfig
    ) external {
        vm.assume(alice != bob);

        LibTestAddOrder.conformConfig(aliceConfig, iInterpreter, iStore);
        aliceConfig.evaluable.bytecode = iParserV2.parse2("_ _:1000 1;:;");

        LibTestAddOrder.conformConfig(bobConfig, iInterpreter, iStore);
        bobConfig.evaluable.bytecode = iParserV2.parse2("_ _:1000 1;:;");

        aliceConfig.validInputs[0].token = address(iToken0);
        aliceConfig.validOutputs[0].token = address(iToken1);

        bobConfig.validInputs[0] = aliceConfig.validOutputs[0];
        bobConfig.validOutputs[0] = aliceConfig.validInputs[0];

        vm.recordLogs();
        vm.prank(alice);
        iRaindex.addOrder4(aliceConfig, new TaskV2[](0));
        OrderV4 memory aliceOrder = LibTestTakeOrder.extractOrderFromLogs(vm.getRecordedLogs());

        vm.recordLogs();
        vm.prank(bob);
        iRaindex.addOrder4(bobConfig, new TaskV2[](0));
        OrderV4 memory bobOrder = LibTestTakeOrder.extractOrderFromLogs(vm.getRecordedLogs());

        ClearConfigV2 memory clearConfig = ClearConfigV2(0, 0, 0, 0, 0, 0);

        vm.mockCall(aliceOrder.validOutputs[0].token, bytes(""), abi.encode(true));
        vm.prank(aliceOrder.owner);
        iRaindex.deposit4(
            aliceOrder.validOutputs[0].token,
            aliceOrder.validOutputs[0].vaultId,
            LibDecimalFloat.packLossless(type(int64).max, 0),
            new TaskV2[](0)
        );
        vm.mockCall(bobOrder.validOutputs[0].token, bytes(""), abi.encode(true));
        vm.prank(bobOrder.owner);
        iRaindex.deposit4(
            bobOrder.validOutputs[0].token,
            bobOrder.validOutputs[0].vaultId,
            LibDecimalFloat.packLossless(type(int64).max, 0),
            new TaskV2[](0)
        );

        // Create a flash borrower.
        Reenteroor borrower = new Reenteroor();
        checkFlashLoanNotRevert(
            borrower,
            abi.encodeWithSelector(
                IRaindexV6.clear3.selector,
                aliceOrder,
                bobOrder,
                clearConfig,
                new SignedContextV1[](0),
                new SignedContextV1[](0)
            ),
            loanAmount
        );
    }
}
