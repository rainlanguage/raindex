#!/bin/bash
# SPDX-License-Identifier: LicenseRef-DCL-1.0
# SPDX-FileCopyrightText: Copyright (c) 2020 Rain Open Source Software Ltd

set -euxo pipefail

mkdir -p meta
forge script --silent ./script/BuildAuthoringMeta.sol
nix shell github:rainlanguage/rain.cli -c rain meta build \
  -i <(cat ./meta/RaindexV6SubParserAuthoringMeta.rain.meta) \
  -m authoring-meta-v2 \
  -t cbor \
  -e deflate \
  -l none \
  -o meta/RaindexV6SubParser.rain.meta
