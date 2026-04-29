// SPDX-License-Identifier: LicenseRef-DCL-1.0
// SPDX-FileCopyrightText: Copyright (c) 2020 Rain Open Source Software Ltd
pragma solidity =0.8.25;

import {GenericPoolRaindexV6ArbOrderTakerTest} from "test/util/abstract/GenericPoolRaindexV6ArbOrderTakerTest.sol";

import {GenericPoolRaindexV6ArbOrderTaker} from "../../src/concrete/arb/GenericPoolRaindexV6ArbOrderTaker.sol";
import {
    IRaindexV6,
    EvaluableV4,
    TakeOrderConfigV4,
    TakeOrdersConfigV5,
    IInterpreterV4,
    IInterpreterStoreV3,
    TaskV2,
    SignedContextV1
} from "rain.raindex.interface/interface/IRaindexV6.sol";
import {LibDecimalFloat} from "rain.math.float/lib/LibDecimalFloat.sol";
import {LibInterpreterDeploy} from "rain.interpreter/lib/deploy/LibInterpreterDeploy.sol";

contract RaindexV6ArbOrderTakerNoOrdersTest is GenericPoolRaindexV6ArbOrderTakerTest {
    /// arb5 MUST revert with NoOrders when given an empty orders array.
    function testArb5NoOrders() external {
        TakeOrderConfigV4[] memory orders = new TakeOrderConfigV4[](0);

        vm.expectRevert(abi.encodeWithSelector(IRaindexV6.NoOrders.selector));
        GenericPoolRaindexV6ArbOrderTaker(iArb)
            .arb5(
                iRaindex,
                TakeOrdersConfigV5({
                minimumIO: LibDecimalFloat.packLossless(0, 0),
                maximumIO: LibDecimalFloat.packLossless(type(int224).max, 0),
                maximumIORatio: LibDecimalFloat.packLossless(type(int224).max, 0),
                IOIsInput: true,
                orders: orders,
                data: abi.encode(iRefundoor, iRefundoor, "")
            }),
                TaskV2({
                evaluable: EvaluableV4(
                    IInterpreterV4(LibInterpreterDeploy.INTERPRETER_DEPLOYED_ADDRESS),
                    IInterpreterStoreV3(LibInterpreterDeploy.STORE_DEPLOYED_ADDRESS),
                    ""
                ),
                signedContext: new SignedContextV1[](0)
            })
            );
    }
}
