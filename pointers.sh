#!/bin/bash

set -euxo pipefail

(cd lib/rain.interpreter/lib/rain.interpreter.interface/lib/rain.math.float && nix develop -c rainix-sol-prelude)
(cd lib/rain.interpreter/lib/rain.interpreter.interface/lib/rain.math.float && nix develop -c rainix-rs-prelude)
(cd lib/rain.interpreter && nix develop -c rainix-sol-prelude)
(cd lib/rain.interpreter && nix develop -c rainix-rs-prelude)
(cd lib/rain.interpreter && nix develop -c i9r-prelude)
(cd lib/rain.interpreter/lib/rain.metadata && nix develop -c rainix-sol-prelude)
(cd lib/rain.interpreter/lib/rain.metadata && nix develop -c rainix-rs-prelude)
(cd lib/rain.interpreter/lib/rain.tofu.erc20-decimals && nix develop -c forge build)

nix develop -c rainix-sol-prelude
nix develop -c rainix-rs-prelude
nix develop -c raindex-prelude

nix develop -c forge script script/BuildPointers.sol