#!/bin/bash

set -euxo pipefail

nix develop -c bash -c '(cd lib/rain.interpreter/lib/rain.interpreter.interface/lib/rain.math.float && rainix-sol-prelude)'
nix develop -c bash -c '(cd lib/rain.interpreter/lib/rain.interpreter.interface/lib/rain.math.float && rainix-rs-prelude)'
nix develop -c bash -c '(cd lib/rain.interpreter && rainix-sol-prelude)'
nix develop -c bash -c '(cd lib/rain.interpreter && rainix-rs-prelude)'
nix develop -c bash -c '(cd lib/rain.interpreter && rainlang-prelude)'
nix develop -c bash -c '(cd lib/rain.interpreter/lib/rain.metadata && rainix-sol-prelude)'
nix develop -c bash -c '(cd lib/rain.interpreter/lib/rain.metadata && rainix-rs-prelude)'
nix develop -c bash -c '(cd lib/rain.interpreter/lib/rain.tofu.erc20-decimals && forge build)'

nix develop -c rainix-sol-prelude
nix develop -c rainix-rs-prelude
nix develop -c raindex-prelude

nix develop -c forge script script/BuildPointers.sol
