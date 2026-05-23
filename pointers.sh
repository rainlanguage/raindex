#!/bin/bash

set -euxo pipefail

nix develop -c forge soldeer install
nix develop -c forge build

nix develop -c raindex-prelude

nix develop -c forge script script/BuildPointers.sol
