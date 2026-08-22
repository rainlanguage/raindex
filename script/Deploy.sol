// SPDX-License-Identifier: LicenseRef-DCL-1.0
// SPDX-FileCopyrightText: Copyright (c) 2020 Rain Open Source Software Ltd
pragma solidity =0.8.25;

import {RainDeployBroadcast} from "rain-deploy-0.1.7/src/abstract/RainDeployBroadcast.sol";
import {RaindexDeploySuites} from "../src/abstract/RaindexDeploySuites.sol";

/// @title Deploy
/// @notice `RainDeployBroadcast` over this repo's one suite declaration:
/// selects a suite by `DEPLOYMENT_SUITE` and deploys it through the Zoltu
/// factory, after anchoring every candidate snapshot to current source.
///
/// The sub-parser's described-by meta emission is NOT part of this broadcast —
/// `RainDeployBroadcast.run` has no post-deploy hook — it moved to
/// `script/DescribeSubParser.sol`, dispatched separately.
contract Deploy is RaindexDeploySuites, RainDeployBroadcast {
    /// @inheritdoc RainDeployBroadcast
    /// @dev Raindex is live on these five networks, not the full
    /// `LibRainDeploy.supportedNetworks()` seven — this is exactly the set the
    /// previous deploy machinery (rain-deploy 0.1.2) broadcast to, so the
    /// override preserves current deploy behaviour. Growing the set to
    /// ethereum/hyperevm is a deliberate future deploy, not a side effect of a
    /// dependency bump.
    function deployNetworks() internal pure override returns (string[] memory) {
        string[] memory networks = new string[](5);
        networks[0] = "arbitrum";
        networks[1] = "base";
        networks[2] = "base_sepolia";
        networks[3] = "flare";
        networks[4] = "polygon";
        return networks;
    }
}
