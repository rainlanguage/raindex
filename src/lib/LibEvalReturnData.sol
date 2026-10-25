// SPDX-License-Identifier: LicenseRef-DCL-1.0
// SPDX-FileCopyrightText: Copyright (c) 2020 Rain Open Source Software Ltd
pragma solidity ^0.8.19;

/// Checks raw `eval4` returndata before decoding, so a malformed return from an
/// order owner's interpreter can be skipped instead of reverting the caller.
library LibEvalReturnData {
    /// True iff `abi.decode(data, (StackItem[], bytes32[]))` does not revert.
    function isDecodable(bytes memory data) internal pure returns (bool) {
        if (data.length < 0x40) {
            return false;
        }
        uint256 stackOffset;
        uint256 kvsOffset;
        assembly ("memory-safe") {
            stackOffset := mload(add(data, 0x20))
            kvsOffset := mload(add(data, 0x40))
        }
        return isArrayInBounds(data, stackOffset) && isArrayInBounds(data, kvsOffset);
    }

    function isArrayInBounds(bytes memory data, uint256 offset) private pure returns (bool) {
        if (offset > type(uint64).max || offset + 0x20 > data.length) {
            return false;
        }
        uint256 arrayLength;
        assembly ("memory-safe") {
            arrayLength := mload(add(add(data, 0x20), offset))
        }
        return arrayLength <= type(uint64).max && offset + 0x20 + arrayLength * 0x20 <= data.length;
    }
}
