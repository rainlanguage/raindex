// SPDX-License-Identifier: LicenseRef-DCL-1.0
// SPDX-FileCopyrightText: Copyright (c) 2020 Rain Open Source Software Ltd
pragma solidity =0.8.25;

import {OrderBookV6ExternalRealTest, console2} from "test/util/abstract/OrderBookV6ExternalRealTest.sol";
import {
    IRaindexV6,
    QuoteV2,
    OrderConfigV4,
    EvaluableV4,
    TaskV2,
    OrderV4,
    SignedContextV1
} from "rain.raindex.interface/interface/IRaindexV6.sol";
import {LibOrderBookDeploy} from "../../../src/lib/deploy/LibOrderBookDeploy.sol";
import {LibTestAddOrder} from "test/util/lib/LibTestAddOrder.sol";
import {IERC20} from "openzeppelin-contracts/contracts/token/ERC20/IERC20.sol";
import {IERC20Metadata} from "openzeppelin-contracts/contracts/token/ERC20/extensions/IERC20Metadata.sol";
import {TokenSelfTrade} from "../../../src/concrete/ob/OrderBookV6.sol";
import {Float, LibDecimalFloat} from "rain.math.float/lib/LibDecimalFloat.sol";
import {Strings} from "openzeppelin-contracts/contracts/utils/Strings.sol";

import {LibFormatDecimalFloat} from "rain.math.float/lib/format/LibFormatDecimalFloat.sol";

/// @title OrderBookV6QuoteTest
contract OrderBookV6QuoteTest is OrderBookV6ExternalRealTest {
    using Strings for address;
    using Strings for uint256;

    using LibDecimalFloat for Float;

    /// Dead orders always eval to false.
    /// forge-config: default.fuzz.runs = 100
    function testQuoteDeadOrder(QuoteV2 memory quoteConfig) external view {
        (bool success, Float maxOutput, Float ioRatio) =
            IRaindexV6(LibOrderBookDeploy.ORDERBOOK_DEPLOYED_ADDRESS).quote2(quoteConfig);
        assert(!success);
        assertTrue(maxOutput.isZero(), "max output");
        assertTrue(ioRatio.isZero(), "io ratio");
    }

    function checkQuote(
        address owner,
        OrderConfigV4 memory config,
        bytes[] memory rainlang,
        Float depositAmount,
        Float[] memory expectedMaxOutput,
        Float[] memory expectedIoRatio
    ) internal {
        checkQuote(
            owner,
            config,
            rainlang,
            depositAmount,
            expectedMaxOutput,
            expectedIoRatio,
            bytes32(uint256(0x01)),
            bytes32(uint256(0x01))
        );
    }

    function checkQuote(
        address owner,
        OrderConfigV4 memory config,
        bytes[] memory rainlang,
        Float depositAmount,
        Float[] memory expectedMaxOutput,
        Float[] memory expectedIoRatio,
        bytes32 outputVaultId,
        bytes32 inputVaultId
    ) internal {
        LibTestAddOrder.conformConfig(config, iInterpreter, iStore);

        uint8 depositDecimals = 12;
        uint256 depositAmount18 = depositAmount.toFixedDecimalLossless(depositDecimals);

        config.validOutputs[0].token = address(iToken0);
        vm.mockCall(
            address(iToken0), abi.encodeWithSelector(IERC20Metadata.decimals.selector), abi.encode(depositDecimals)
        );

        config.validInputs[0].token = address(iToken1);
        vm.mockCall(address(iToken1), abi.encodeWithSelector(IERC20Metadata.decimals.selector), abi.encode(6));

        if (outputVaultId == bytes32(0)) {
            config.validOutputs[0].vaultId = bytes32(0);
            mockVault0Output(address(iToken0), owner, depositAmount18);
        } else {
            vm.mockCall(
                address(iToken0),
                abi.encodeWithSelector(
                    IERC20.transferFrom.selector, owner, LibOrderBookDeploy.ORDERBOOK_DEPLOYED_ADDRESS, depositAmount18
                ),
                abi.encode(true)
            );
            vm.prank(owner);
            IRaindexV6(LibOrderBookDeploy.ORDERBOOK_DEPLOYED_ADDRESS)
                .deposit4(config.validOutputs[0].token, config.validOutputs[0].vaultId, depositAmount, new TaskV2[](0));
        }
        if (inputVaultId == bytes32(0)) {
            config.validInputs[0].vaultId = bytes32(0);
            mockVault0Input(address(iToken1), owner, 0);
        }

        for (uint256 i = 0; i < rainlang.length; i++) {
            config.evaluable.bytecode = iParserV2.parse2(rainlang[i]);
            vm.prank(owner);
            IRaindexV6(LibOrderBookDeploy.ORDERBOOK_DEPLOYED_ADDRESS).addOrder4(config, new TaskV2[](0));

            OrderV4 memory order = OrderV4({
                owner: owner,
                evaluable: config.evaluable,
                validInputs: config.validInputs,
                validOutputs: config.validOutputs,
                nonce: config.nonce
            });

            QuoteV2 memory quoteConfig =
                QuoteV2({order: order, inputIOIndex: 0, outputIOIndex: 0, signedContext: new SignedContextV1[](0)});
            (bool success, Float maxOutput, Float ioRatio) =
                IRaindexV6(LibOrderBookDeploy.ORDERBOOK_DEPLOYED_ADDRESS).quote2(quoteConfig);
            assert(success);

            assertTrue(maxOutput.eq(expectedMaxOutput[i]), "max output");
            assertTrue(ioRatio.eq(expectedIoRatio[i]), "io ratio");
        }
    }

    function checkQuote(
        address owner,
        OrderConfigV4 memory config,
        bytes memory rainlang,
        Float depositAmount,
        Float expectedMaxOutput,
        Float expectedIoRatio
    ) internal {
        bytes[] memory rainlangArray = new bytes[](1);
        rainlangArray[0] = rainlang;

        Float[] memory expectedMaxOutputArray = new Float[](1);
        expectedMaxOutputArray[0] = expectedMaxOutput;

        Float[] memory expectedIoRatioArray = new Float[](1);
        expectedIoRatioArray[0] = expectedIoRatio;

        checkQuote(owner, config, rainlangArray, depositAmount, expectedMaxOutputArray, expectedIoRatioArray);
    }

    /// forge-config: default.fuzz.runs = 100
    function testQuoteSimple(address owner, OrderConfigV4 memory config, uint256 depositAmount18) external {
        depositAmount18 = bound(depositAmount18, 1e18, uint256(int256(type(int224).max)) / 1e6);
        Float depositAmount = LibDecimalFloat.fromFixedDecimalLosslessPacked(depositAmount18, 12);
        checkQuote(
            owner,
            config,
            "_ _:1 2;",
            depositAmount,
            LibDecimalFloat.packLossless(1, 0),
            LibDecimalFloat.packLossless(2, 0)
        );
    }

    /// The output will be maxed at the deposit in the vault.
    /// forge-config: default.fuzz.runs = 100
    function testQuoteMaxOutput(address owner, OrderConfigV4 memory config, uint256 depositAmount18) external {
        depositAmount18 = bound(depositAmount18, 1, 1e12);
        Float depositAmount = LibDecimalFloat.fromFixedDecimalLosslessPacked(depositAmount18, 12);
        checkQuote(owner, config, "_ _:1 2;:;", depositAmount, depositAmount, LibDecimalFloat.packLossless(2, 0));
    }

    /// Can access context.
    /// forge-config: default.fuzz.runs = 100
    function testQuoteContextSender(address owner, OrderConfigV4 memory config, uint256 depositAmount18) external {
        // Max amount needs to be small enough to be scaled up to 18 decimals
        // from 12 decimals.
        depositAmount18 = bound(depositAmount18, 1e18, uint256(int256(type(int224).max)) / 1e6);

        Float depositAmount = LibDecimalFloat.fromFixedDecimalLosslessPacked(depositAmount18, 12);

        string memory usingWordsFrom = string.concat("using-words-from ", address(iSubParser).toHexString(), "\n");

        bytes[] memory rainlang = new bytes[](10);
        // quote msg.sender
        rainlang[0] = "_ _:1 context<0 0>();:;";
        // orderbook
        rainlang[1] = "_ _:1 context<0 1>();:;";
        // // We can't easily check order hash with this setup
        // rainlang[2] = "_ _:1 context<1 0>();:;";
        // quote order owner
        rainlang[2] = "_ _:1 context<1 1>();:;";
        // quote order counterparty (will be quoter)
        rainlang[3] = "_ _:1 context<1 2>();:;";
        // calculations won't be in context for quote
        // inputs
        // vault input token
        rainlang[4] = "_ _:1 context<3 0>();:;";
        // vault input token decimals
        rainlang[5] = bytes(
            string.concat(
                usingWordsFrom,
                ":ensure(equal-to(input-token-decimals() context<3 1>()) \"input decimals\"),",
                ":ensure(equal-to(input-token-decimals() 6) \"input decimals literal\"),",
                "_ _:1 context<3 1>();:;"
            )
        );
        // vault io vault id
        // not easy to test with this setup
        // rainlang[6] = "_ _:1 context<3 2>();:;";
        // input vault balance before
        rainlang[6] = bytes(
            string.concat(
                usingWordsFrom,
                ":ensure(equal-to(input-vault-before() context<3 3>()) \"input vault before\"),",
                ":ensure(equal-to(input-vault-before() 0) \"input vault before literal\"),",
                "_ _:1 context<3 3>();:;"
            )
        );
        // outputs
        // vault output token
        rainlang[7] = "_ _:1 context<4 0>();:;";
        // vault output token decimals
        rainlang[8] = bytes(
            string.concat(
                usingWordsFrom,
                ":ensure(equal-to(output-token-decimals() context<4 1>()) \"output decimals\"),",
                ":ensure(equal-to(output-token-decimals() 12) \"output decimals literal\"),",
                "_ _:1 context<4 1>();:;"
            )
        );
        // vault io vault id
        // not easy to test with this setup
        // rainlang[9] = "_ _:1 context<4 2>();:;";
        // output vault balance before
        rainlang[9] = bytes(
            string.concat(
                usingWordsFrom,
                ":ensure(equal-to(output-vault-before() context<4 3>()) \"output vault before\"),",
                ":ensure(equal-to(output-vault-before() ",
                depositAmount18.toString(),
                "e-12) \"output vault before literal\"),",
                "_ _:1 context<4 3>();:;"
            )
        );

        Float[] memory expectedMaxOutput = new Float[](10);
        expectedMaxOutput[0] = LibDecimalFloat.packLossless(1, 0);
        expectedMaxOutput[1] = LibDecimalFloat.packLossless(1, 0);
        expectedMaxOutput[2] = LibDecimalFloat.packLossless(1, 0);
        expectedMaxOutput[3] = LibDecimalFloat.packLossless(1, 0);
        expectedMaxOutput[4] = LibDecimalFloat.packLossless(1, 0);
        expectedMaxOutput[5] = LibDecimalFloat.packLossless(1, 0);
        expectedMaxOutput[6] = LibDecimalFloat.packLossless(1, 0);
        expectedMaxOutput[7] = LibDecimalFloat.packLossless(1, 0);
        expectedMaxOutput[8] = LibDecimalFloat.packLossless(1, 0);
        expectedMaxOutput[9] = LibDecimalFloat.packLossless(1, 0);

        Float[] memory expectedIoRatio = new Float[](10);
        expectedIoRatio[0] = Float.wrap(bytes32(uint256(uint160(address(this)))));
        expectedIoRatio[1] = Float.wrap(bytes32(uint256(uint160(LibOrderBookDeploy.ORDERBOOK_DEPLOYED_ADDRESS))));
        expectedIoRatio[2] = Float.wrap(bytes32(uint256(uint160(owner))));
        expectedIoRatio[3] = Float.wrap(bytes32(uint256(uint160(address(this)))));
        expectedIoRatio[4] = Float.wrap(bytes32(uint256(uint160(address(iToken1)))));
        // Input decimals scaled to 18 fixed point value.
        expectedIoRatio[5] = LibDecimalFloat.packLossless(6, 0);
        expectedIoRatio[6] = LibDecimalFloat.packLossless(0, 0);
        expectedIoRatio[7] = Float.wrap(bytes32(uint256(uint160(address(iToken0)))));
        expectedIoRatio[8] = LibDecimalFloat.fromFixedDecimalLosslessPacked(12e18, 18);
        // Output decimals scaled to 18 fixed point value from 12.
        expectedIoRatio[9] = LibDecimalFloat.fromFixedDecimalLosslessPacked(depositAmount18 * 1e6, 18);

        checkQuote(owner, config, rainlang, depositAmount, expectedMaxOutput, expectedIoRatio);
    }

    /// forge-config: default.fuzz.runs = 100
    function testQuoteSimpleBothVaultIdZero(address owner, OrderConfigV4 memory config, uint256 depositAmount18)
        external
    {
        depositAmount18 = bound(depositAmount18, 1e18, uint256(int256(type(int224).max)) / 1e6);
        Float depositAmount = LibDecimalFloat.fromFixedDecimalLosslessPacked(depositAmount18, 12);

        bytes[] memory rainlangArray = new bytes[](1);
        rainlangArray[0] = "_ _:1 2;";

        Float[] memory expectedMaxOutputArray = new Float[](1);
        expectedMaxOutputArray[0] = LibDecimalFloat.packLossless(1, 0);

        Float[] memory expectedIoRatioArray = new Float[](1);
        expectedIoRatioArray[0] = LibDecimalFloat.packLossless(2, 0);

        checkQuote(
            owner,
            config,
            rainlangArray,
            depositAmount,
            expectedMaxOutputArray,
            expectedIoRatioArray,
            bytes32(0),
            bytes32(0)
        );
    }
}
