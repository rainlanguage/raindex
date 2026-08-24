// SPDX-License-Identifier: LicenseRef-DCL-1.0
// SPDX-FileCopyrightText: Copyright (c) 2020 Rain Open Source Software Ltd
pragma solidity =0.8.25;

import {Test, console2} from "forge-std-1.16.1/src/Test.sol";
import {IERC20} from "@openzeppelin-contracts-5.6.1/token/ERC20/IERC20.sol";
import {IERC4626} from "@openzeppelin-contracts-5.6.1/interfaces/IERC4626.sol";
import {
    IRaindexV6,
    QuoteV2,
    OrderV4,
    SignedContextV1,
    TakeOrderConfigV4,
    TakeOrdersConfigV5,
    TaskV2,
    Float
} from "raindex-interface-0.1.1/src/interface/IRaindexV6.sol";
import {LibOrder} from "../../../src/lib/LibOrder.sol";
import {LibDecimalFloat} from "rain-math-float-0.1.1/src/lib/LibDecimalFloat.sol";

/// @title RaindexV6St0xNavForkTest
/// @notice Base fork coverage for st0x-fixed-spread-v6 vault NAV bind:
/// quote+take succeeds when live NAV matches signed slot 9; take reverts after
/// a dividend-style NAV step between quote and take.
///
/// Run:
/// `BASE_MAINNET_RPC_URL=<rpc> forge test --match-contract RaindexV6St0xNavForkTest -vvv`
contract RaindexV6St0xNavForkTest is Test {
    using LibOrder for OrderV4;
    using LibDecimalFloat for Float;

    address internal constant RAINDEX = 0xe522cB4a5fCb2eb31a52Ff41a4653d85A4fd7C9D;
    address internal constant USDC = 0x833589fCD6eDb6E08f4c7C32D4f71b54bdA02913;
    address internal constant WT_VAULT = 0x997baE3EC193a249596d3708C3fAB7C501Bb8a53;
    address internal constant ORACLE_SIGNER = 0xdB665B5ef8Bd04Fd977bB3A64790eaa52749ACcc;

    bytes32 internal constant EXPECTED_ORDER_HASH =
        0x153efcdae1ec00a07849110f1107ccc49d4289b94729670a297db1138390e4a8;

    /// Latest tip provided for this fixture.
    uint256 internal constant FORK_BLOCK = 0x300ffbd;

    /// Signed frame publish / expiry (unix). Warp into this window so session
    /// and expiry guards do not false-fail.
    uint256 internal constant ORACLE_PUBLISH_TIME = 0x6a8c5c5b;

    /// Deposit enough shares for a small take (1e-6 shares maximumIO uses 1e12 raw).
    uint256 internal constant DEPOSIT_SHARES = 1e15;

    bytes internal constant ORDER_BYTES =
        hex"0000000000000000000000000000000000000000000000000000000000000020000000000000000000000000669845c29d9b1a64fff66a55aa13eb4adb889a8800000000000000000000000000000000000000000000000000000000000000a000000000000000000000000000000000000000000000000000000000000005e0000000000000000000000000000000000000000000000000000000000000064004df0f10b937664aad05704bf3443b53f6ee951aa022466693b318af4243ea6b0000000000000000000000003bf9bd9da4784f75c92317e61c68493ecc9aabdc0000000000000000000000001aa775533e28b1d843e1a589034984e3a62005dc000000000000000000000000000000000000000000000000000000000000006000000000000000000000000000000000000000000000000000000000000004a50000000000000000000000000000000000000000000000000000000000000018000000000000000000000000db665b5ef8bd04fd977bb3a64790eaa52749accc0000000000000000556e74727573746564206f7261636c65207369676e6572f70000000000000000000000000000000000000000000000000000000000000006004f7261636c6520736368656d612076657273696f6e206d69736d61746368fe000000004f7261636c6520696e70757420746f6b656e206d69736d61746368fb0000004f7261636c65206f757470757420746f6b656e206d69736d61746368fc00000000000000000000000000000000000000000000000000000000727468e3000000000000000000000000000000000000000000007072656d61726b6574e90000000000000000000000000000000000000000006166746572686f757273ea0000004e6f7420696e20616374697665206d61726b65742073657373696f6efc00000000000000000000004265666f72652073657373696f6e207374617274f4000000000000000000000000000041667465722073657373696f6e20656e64f100000000005072696365206265666f72652073657373696f6e207374617274fa0000000000000000000000000050617374206f7261636c6520657870697279f2000000000000000000000000997bae3ec193a249596d3708c3fab7c501bb8a530000000000000000000000000000000000000000000000000000000000000001000000000000005661756c74204e415620726174696f206d69736d61746368f8000000000000000000000000000000000000000000000000000000000000000000000000000000000000004f7261636c65207072696365206973207a65726ff4fffffffd0000000000000000000000000000000000000000000000000000ef9700000000000050726963652062656c6f77206d696e696d756d20626f756e64f9fffffffe00000000000000000000000000000000000000000000000000017f5800000000000050726963652061626f7665206d6178696d756d20626f756e64f9000000000000000000000000d69dc3d58a7c875117f9c7cecf4f1a7f3ca472540000000000000000000000000000000000000000000000000000000000000165020000015c560f000c0110000101100000031000051f1200001e020000031000060110000301100002001000001f1200001e0200000310010603100206031003060310040603100506031006060310070603100806031009060110000403100003001000061f1200001e0200000110000503100004001000071f1200001e0200000110000901100008001000031f12000001100007001000031f12000001100006001000031f1200001b1300001e0200000110000a0010000419100000221200001e0200000110000b0010000519100000261200001e0200000110000c0010000400100002221200001e0200000110000d0010000819100000261200001e020000011000100110000f0110000e02120017001000091f1200001e020000011000120110001100100001211200001e020000011000140110001300100001221200001e020000011000160110001500100001261200001e0200003610000000100001000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000001000000000000000000000000833589fcd6edb6e08f4c7c32d4f71b54bda02913e261ac1a15a788b5bffa2e413ec64b428e230fc64be8931670a996a7e718342f0000000000000000000000000000000000000000000000000000000000000001000000000000000000000000997bae3ec193a249596d3708c3fab7c501bb8a53e261ac1a15a788b5bffa2e413ec64b428e230fc64be8931670a996a7e718342f";

    function setUp() public {
        string memory rpc = vm.envOr("BASE_MAINNET_RPC_URL", string("https://mainnet.base.org"));
        vm.createSelectFork(rpc, FORK_BLOCK);
        // Frame expiry is only ~20s after publish; pin time inside the window.
        vm.warp(ORACLE_PUBLISH_TIME);
    }

    function testQuoteAndTakeSucceedsWhenNavMatches() external {
        OrderV4 memory order = _order();
        SignedContextV1[] memory signedContext = _signedContext();

        _depositOutputShares(order, DEPOSIT_SHARES);

        (bool success, Float maxOutput, Float ioRatio) = _quote(order, signedContext);
        assertTrue(success, "quote2 should succeed when NAV matches");
        assertFalse(maxOutput.isZero(), "maxOutput");
        assertFalse(ioRatio.isZero(), "ioRatio");

        _fundUsdcAndApprove(1_000_000e6);
        (Float totalIn, Float totalOut) = IRaindexV6(RAINDEX).takeOrders4(_takeConfig(order, signedContext));
        assertFalse(totalIn.isZero(), "taker should receive shares");
        assertFalse(totalOut.isZero(), "taker should pay USDC");
        console2.log("take ok");
    }

    function testTakeRevertsWhenNavStepsAfterQuote() external {
        OrderV4 memory order = _order();
        SignedContextV1[] memory signedContext = _signedContext();

        _depositOutputShares(order, DEPOSIT_SHARES);

        (bool success,,) = _quote(order, signedContext);
        assertTrue(success, "quote2 should succeed before NAV step");

        uint256 navBefore = IERC4626(WT_VAULT).convertToAssets(1e18);
        _donateUnderlyingToVault(/* amount */ 1_000e18);
        uint256 navAfter = IERC4626(WT_VAULT).convertToAssets(1e18);
        assertTrue(navAfter != navBefore, "donate must change convertToAssets(1e18)");
        console2.log("navBefore", navBefore);
        console2.log("navAfter", navAfter);

        _fundUsdcAndApprove(1_000_000e6);
        vm.expectRevert("Vault NAV ratio mismatch");
        IRaindexV6(RAINDEX).takeOrders4(_takeConfig(order, signedContext));
    }

    function _order() internal pure returns (OrderV4 memory order) {
        order = abi.decode(ORDER_BYTES, (OrderV4));
        require(order.hash() == EXPECTED_ORDER_HASH, "order hash mismatch");
    }

    function _signedContext() internal pure returns (SignedContextV1[] memory signedContext) {
        signedContext = new SignedContextV1[](1);
        signedContext[0].signer = ORACLE_SIGNER;
        signedContext[0].context = new bytes32[](10);
        signedContext[0].context[0] = 0x0000000000000000000000000000000000000000000000000000000000000006;
        signedContext[0].context[1] = 0xffffffc018e7fa9c7f8d3018574d5e91ad0eabcca9939de04866300000000000;
        signedContext[0].context[2] = 0x000000000000000000000000000000000000000000000000000000006a8c5c5b;
        signedContext[0].context[3] = 0x00000000000000000000000000000000000000000000000000000000727468e3;
        signedContext[0].context[4] = 0x000000000000000000000000000000000000000000000000000000006a8c4758;
        signedContext[0].context[5] = 0x000000000000000000000000000000000000000000000000000000006a8ca2c0;
        signedContext[0].context[6] = 0x000000000000000000000000833589fcd6edb6e08f4c7c32d4f71b54bda02913;
        signedContext[0].context[7] = 0x000000000000000000000000997bae3ec193a249596d3708c3fab7c501bb8a53;
        signedContext[0].context[8] = 0x000000000000000000000000000000000000000000000000000000006a8c5c6f;
        signedContext[0].context[9] = 0xffffffee00000000000000000000000000000000000000000de0b6b3a7640000;
        signedContext[0].signature =
            hex"218e9a6c364b1d0c6773dc620b6f5b3f7c5bc05368d23bb896a6d216514cda7e420690a8f9b3894a66572edc3f9be8771c95433c16523bc3f521148b7a4657c01c";
    }

    function _quote(OrderV4 memory order, SignedContextV1[] memory signedContext)
        internal
        view
        returns (bool success, Float maxOutput, Float ioRatio)
    {
        return IRaindexV6(RAINDEX).quote2(
            QuoteV2({order: order, inputIOIndex: 0, outputIOIndex: 0, signedContext: signedContext})
        );
    }

    function _takeConfig(OrderV4 memory order, SignedContextV1[] memory signedContext)
        internal
        pure
        returns (TakeOrdersConfigV5 memory)
    {
        TakeOrderConfigV4[] memory orders = new TakeOrderConfigV4[](1);
        orders[0] = TakeOrderConfigV4({order: order, inputIOIndex: 0, outputIOIndex: 0, signedContext: signedContext});
        // Small take: 1e-6 whole shares (18 decimals) as maximum input.
        return TakeOrdersConfigV5({
            minimumIO: LibDecimalFloat.packLossless(0, 0),
            maximumIO: LibDecimalFloat.fromFixedDecimalLosslessPacked(1e12, 18),
            maximumIORatio: LibDecimalFloat.packLossless(type(int224).max, 0),
            IOIsInput: true,
            orders: orders,
            data: ""
        });
    }

    function _depositOutputShares(OrderV4 memory order, uint256 shareAmount) internal {
        address owner = order.owner;
        bytes32 vaultId = order.validOutputs[0].vaultId;
        deal(WT_VAULT, owner, shareAmount);
        vm.startPrank(owner);
        IERC20(WT_VAULT).approve(RAINDEX, shareAmount);
        IRaindexV6(RAINDEX).deposit4(
            WT_VAULT, vaultId, LibDecimalFloat.fromFixedDecimalLosslessPacked(shareAmount, 18), new TaskV2[](0)
        );
        vm.stopPrank();
    }

    function _fundUsdcAndApprove(uint256 amount) internal {
        deal(USDC, address(this), amount);
        IERC20(USDC).approve(RAINDEX, type(uint256).max);
    }

    /// Dividend-style NAV step: push underlying into the vault without minting shares.
    function _donateUnderlyingToVault(uint256 amount) internal {
        address asset = IERC4626(WT_VAULT).asset();
        deal(asset, address(this), amount);
        IERC20(asset).transfer(WT_VAULT, amount);
    }
}
