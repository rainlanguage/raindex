// SPDX-License-Identifier: LicenseRef-DCL-1.0
// SPDX-FileCopyrightText: Copyright (c) 2020 Rain Open Source Software Ltd
pragma solidity =0.8.25;

import {Test, Vm, console2} from "forge-std/Test.sol";
import {REVERTING_MOCK_BYTECODE} from "test/util/lib/LibTestConstants.sol";
import {IRaindexV6Stub} from "test/util/abstract/IRaindexV6Stub.sol";
import {IInterpreterStoreV3} from "rain.interpreter.interface/interface/IInterpreterStoreV3.sol";
import {IParserV2} from "rain.interpreter.interface/interface/IParserV2.sol";
import {
    IRaindexV6,
    IInterpreterV4,
    TaskV2,
    EvaluableV4,
    SignedContextV1
} from "rain.raindex.interface/interface/IRaindexV6.sol";
import {IERC20} from "openzeppelin-contracts/contracts/token/ERC20/IERC20.sol";
import {OrderBookV6SubParser} from "../../../src/concrete/parser/OrderBookV6SubParser.sol";
import {IERC20Metadata} from "openzeppelin-contracts/contracts/token/ERC20/extensions/IERC20Metadata.sol";
import {LibDecimalFloat, Float} from "rain.math.float/lib/LibDecimalFloat.sol";
import {LibTOFUTokenDecimals} from "rain.tofu.erc20-decimals/lib/LibTOFUTokenDecimals.sol";
import {LibInterpreterDeploy} from "rain.interpreter/lib/deploy/LibInterpreterDeploy.sol";
import {LibRainDeploy} from "rain.deploy/lib/LibRainDeploy.sol";
import {LibOrderBookDeploy} from "../../../src/lib/deploy/LibOrderBookDeploy.sol";
import {LibEtchOrderBook} from "test/util/lib/LibEtchOrderBook.sol";

abstract contract OrderBookV6ExternalRealTest is Test, IRaindexV6Stub {
    IInterpreterV4 internal immutable iInterpreter;
    IInterpreterStoreV3 internal immutable iStore;
    IParserV2 internal immutable iParserV2;
    IERC20 internal immutable iToken0;
    IERC20 internal immutable iToken1;
    OrderBookV6SubParser internal immutable iSubParser;

    constructor() {
        LibRainDeploy.etchZoltuFactory(vm);
        LibRainDeploy.deployZoltu(LibTOFUTokenDecimals.TOFU_DECIMALS_EXPECTED_CREATION_CODE);

        LibInterpreterDeploy.etchRainlang(vm);
        LibEtchOrderBook.etchOrderBook(vm);

        iInterpreter = IInterpreterV4(LibInterpreterDeploy.INTERPRETER_DEPLOYED_ADDRESS);
        iStore = IInterpreterStoreV3(LibInterpreterDeploy.STORE_DEPLOYED_ADDRESS);
        iParserV2 = IParserV2(LibInterpreterDeploy.EXPRESSION_DEPLOYER_DEPLOYED_ADDRESS);

        iToken0 = IERC20(address(uint160(uint256(keccak256("token0.rain.test")))));
        vm.etch(address(iToken0), REVERTING_MOCK_BYTECODE);
        vm.mockCall(address(iToken0), abi.encodeWithSelector(IERC20Metadata.decimals.selector), abi.encode(18));

        iToken1 = IERC20(address(uint160(uint256(keccak256("token1.rain.test")))));
        vm.etch(address(iToken1), REVERTING_MOCK_BYTECODE);
        vm.mockCall(address(iToken1), abi.encodeWithSelector(IERC20Metadata.decimals.selector), abi.encode(18));

        iSubParser = OrderBookV6SubParser(LibOrderBookDeploy.SUB_PARSER_DEPLOYED_ADDRESS);
    }

    function assumeEtchable(address account) internal view {
        assumeNotPrecompile(account);
        vm.assume(account != address(iInterpreter));
        vm.assume(account != address(iStore));
        vm.assume(account != address(iParserV2));
        vm.assume(account != LibInterpreterDeploy.PARSER_DEPLOYED_ADDRESS);

        vm.assume(account != LibOrderBookDeploy.ORDERBOOK_DEPLOYED_ADDRESS);
        vm.assume(account != address(iToken0));
        vm.assume(account != address(iToken1));
        vm.assume(account != LibOrderBookDeploy.SUB_PARSER_DEPLOYED_ADDRESS);

        vm.assume(account != address(this));
        vm.assume(account != address(vm));
        // The console.
        vm.assume(account != address(0x000000000000000000636F6e736F6c652e6c6f67));
    }

    function evalsToActions(bytes[] memory evals) internal view returns (TaskV2[] memory) {
        TaskV2[] memory actions = new TaskV2[](evals.length);
        for (uint256 i = 0; i < evals.length; i++) {
            actions[i] = TaskV2(EvaluableV4(iInterpreter, iStore, iParserV2.parse2(evals[i])), new SignedContextV1[](0));
        }
        return actions;
    }

    /// Mock the balanceOf and allowance for a vault 0 output vault (owner
    /// sells from wallet). Also mocks transferFrom for pullTokens with any
    /// amount.
    function mockVault0Output(address token, address owner, uint256 depositAmount18) internal {
        vm.mockCall(token, abi.encodeWithSelector(IERC20.balanceOf.selector, owner), abi.encode(depositAmount18));
        vm.mockCall(
            token,
            abi.encodeWithSelector(IERC20.allowance.selector, owner, LibOrderBookDeploy.ORDERBOOK_DEPLOYED_ADDRESS),
            abi.encode(depositAmount18)
        );
        vm.mockCall(
            token,
            abi.encodeWithSelector(IERC20.transferFrom.selector, owner, LibOrderBookDeploy.ORDERBOOK_DEPLOYED_ADDRESS),
            abi.encode(true)
        );
    }

    /// Mock the balanceOf and allowance (zero) for a vault 0 input vault
    /// (owner receives to wallet). Mocks the transfer for pushTokens if
    /// the expected amount is non-zero.
    function mockVault0Input(address token, address owner, uint256 expectAmount18) internal {
        vm.mockCall(token, abi.encodeWithSelector(IERC20.balanceOf.selector, owner), abi.encode(0));
        vm.mockCall(
            token,
            abi.encodeWithSelector(IERC20.allowance.selector, owner, LibOrderBookDeploy.ORDERBOOK_DEPLOYED_ADDRESS),
            abi.encode(0)
        );
        if (expectAmount18 > 0) {
            vm.mockCall(
                token, abi.encodeWithSelector(IERC20.transfer.selector, owner, expectAmount18), abi.encode(true)
            );
        }
    }
}
