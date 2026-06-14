// SPDX-License-Identifier: LicenseRef-DCL-1.0
// SPDX-FileCopyrightText: Copyright (c) 2020 Rain Open Source Software Ltd
pragma solidity =0.8.25;

import {GenericPoolRaindexV6ArbOrderTakerTest} from "test/util/abstract/GenericPoolRaindexV6ArbOrderTakerTest.sol";

import {GenericPoolRaindexV6ArbOrderTaker} from "../../src/concrete/arb/GenericPoolRaindexV6ArbOrderTaker.sol";
import {BadRaindex} from "../../src/abstract/RaindexV6ArbOrderTaker.sol";
import {LibRaindexDeploy} from "../../src/lib/deploy/LibRaindexDeploy.sol";
import {
    IRaindexV6,
    OrderV4,
    EvaluableV4,
    TakeOrderConfigV4,
    TakeOrdersConfigV5,
    IInterpreterV4,
    IInterpreterStoreV3,
    TaskV2,
    SignedContextV1
} from "raindex-interface-0.1.1/src/interface/IRaindexV6.sol";
import {LibDecimalFloat} from "rain-math-float-0.1.1/src/lib/LibDecimalFloat.sol";
import {LibInterpreterDeploy} from "rainlang-0.1.5/src/lib/deploy/LibInterpreterDeploy.sol";

contract RaindexV6ArbOrderTakerBadRaindexTest is GenericPoolRaindexV6ArbOrderTakerTest {
    /// arb5 MUST revert BadRaindex when `raindex` is not the trusted deterministic
    /// deployment, before any token approval is granted. A caller-supplied
    /// untrusted `raindex` would otherwise receive an unlimited approval and could
    /// drain the arb; the guard rejects it up front (the take-orders path is never
    /// reached). The non-revert control is exercised by every other arb5 test,
    /// which calls the etched raindex at RAINDEX_DEPLOYED_ADDRESS.
    function testArb5RevertsUntrustedRaindex(address badRaindex) external {
        vm.assume(badRaindex != LibRaindexDeploy.RAINDEX_DEPLOYED_ADDRESS);

        // A single valid order so the NoOrders guard passes and execution reaches
        // the raindex-trust check.
        OrderV4 memory order;
        TakeOrderConfigV4[] memory orders = buildTakeOrderConfig(order, 0, 0);

        vm.expectRevert(abi.encodeWithSelector(BadRaindex.selector, badRaindex));
        GenericPoolRaindexV6ArbOrderTaker(iArb)
            .arb5(
                IRaindexV6(badRaindex),
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
