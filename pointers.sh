#!/bin/bash

set -euxo pipefail

nix develop -c forge soldeer install

nix develop -c rainix-sol-prelude
nix develop -c rainix-rs-prelude
nix develop -c raindex-prelude

nix develop -c forge script script/BuildPointers.sol
