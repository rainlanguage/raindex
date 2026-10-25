// SPDX-License-Identifier: LicenseRef-DCL-1.0
// SPDX-FileCopyrightText: Copyright (c) 2020 Rain Open Source Software Ltd
pragma solidity =0.8.25;

import {Test} from "forge-std-1.17.0/src/Test.sol";
import {StackItem} from "rainlang-interface-0.2.13/src/interface/IInterpreterV4.sol";
import {LibEvalReturnData} from "src/lib/LibEvalReturnData.sol";

/// `isDecodable` agrees with the Solidity ABI decoder: true exactly when
/// `abi.decode(data, (StackItem[], bytes32[]))` does not revert.
contract LibEvalReturnDataIsDecodableTest is Test {
    function externalDecode(bytes memory data) external pure returns (StackItem[] memory, bytes32[] memory) {
        return abi.decode(data, (StackItem[], bytes32[]));
    }

    function decodes(bytes memory data) internal view returns (bool) {
        try this.externalDecode(data) {
            return true;
        } catch {
            return false;
        }
    }

    function checkAgrees(bytes memory data) internal view {
        assertEq(LibEvalReturnData.isDecodable(data), decodes(data));
    }

    function testIsDecodableRandomBytes(bytes memory data) external view {
        checkAgrees(data);
    }

    function testIsDecodableStructured(
        uint256 words,
        uint256 stackOffset,
        uint256 kvsOffset,
        uint256 stackLength,
        uint256 kvsLength,
        uint256 trailingBytes
    ) external view {
        words = bound(words, 0, 12);
        trailingBytes = bound(trailingBytes, 0, 31);
        bytes memory data = new bytes(words * 0x20 + trailingBytes);
        stackOffset = boundOrRaw(stackOffset);
        kvsOffset = boundOrRaw(kvsOffset);
        stackLength = boundOrRaw(stackLength);
        kvsLength = boundOrRaw(kvsLength);
        writeWord(data, 0, stackOffset);
        writeWord(data, 0x20, kvsOffset);
        if (stackOffset < data.length) writeWord(data, stackOffset, stackLength);
        if (kvsOffset < data.length) writeWord(data, kvsOffset, kvsLength);
        checkAgrees(data);
    }

    function boundOrRaw(uint256 value) internal pure returns (uint256) {
        if (value % 4 == 0) return value;
        if (value % 4 == 1) return type(uint64).max + (value % 3);
        return (value >> 8) % 0x1a0;
    }

    function writeWord(bytes memory data, uint256 offset, uint256 value) internal pure {
        for (uint256 i = 0; i < 0x20 && offset + i < data.length; i++) {
            data[offset + i] = bytes32(value)[i];
        }
    }

    function testIsDecodableEmpty() external view {
        checkAgrees("");
        assertFalse(LibEvalReturnData.isDecodable(""));
    }

    function testIsDecodableWellFormed(StackItem[] memory stack, bytes32[] memory kvs) external view {
        bytes memory data = abi.encode(stack, kvs);
        assertTrue(LibEvalReturnData.isDecodable(data));
        checkAgrees(data);
    }

    function testIsDecodableOneWordShort(StackItem[] memory stack, bytes32[] memory kvs) external view {
        bytes memory data = abi.encode(stack, kvs);
        assembly ("memory-safe") {
            mstore(data, sub(mload(data), 0x20))
        }
        checkAgrees(data);
        assertFalse(LibEvalReturnData.isDecodable(data));
    }

    function testIsDecodableOffsetAtUint64Max() external view {
        bytes memory data = abi.encode(uint256(type(uint64).max), uint256(0x40), uint256(0));
        checkAgrees(data);
        assertFalse(LibEvalReturnData.isDecodable(data));
    }

    function testIsDecodableLengthAboveUint64Max() external view {
        bytes memory data = abi.encode(uint256(0x40), uint256(0x40), uint256(type(uint64).max) + 1);
        checkAgrees(data);
        assertFalse(LibEvalReturnData.isDecodable(data));
    }

    function testIsDecodableSharedLengthWord() external view {
        bytes memory data = abi.encode(uint256(0x40), uint256(0x40), uint256(0));
        checkAgrees(data);
        assertTrue(LibEvalReturnData.isDecodable(data));
    }
}
