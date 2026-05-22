#!/bin/bash

set -euxo pipefail

echo "Starting project setup..."

keep=(
  -k CI_DEPLOY_SEPOLIA_RPC_URL
  -k CI_FORK_SEPOLIA_DEPLOYER_ADDRESS
  -k CI_FORK_SEPOLIA_BLOCK_NUMBER
  -k CI_DEPLOY_ARBITRUM_RPC_URL
  -k CI_DEPLOY_BASE_RPC_URL
  -k CI_DEPLOY_BASE_SEPOLIA_RPC_URL
  -k CI_DEPLOY_FLARE_RPC_URL
  -k CI_DEPLOY_POLYGON_RPC_URL
  -k CI_SEPOLIA_METABOARD_URL
  -k RPC_URL_ETHEREUM_FORK
  -k COMMIT_SHA
  -k DEPLOYMENT_KEY
  -k PUBLIC_WALLETCONNECT_PROJECT_ID
)

git submodule update --init --recursive

nix develop -c forge soldeer install

nix develop -i "${keep[@]}" -c bash \
  -c '(cd lib/rain.interpreter/lib/rain.interpreter.interface/lib/rain.math.float && rainix-rs-prelude)'

nix develop -i "${keep[@]}" -c bash -c '(cd lib/rain.interpreter && rainix-rs-prelude)'
(cd lib/rain.interpreter && nix develop -i "${keep[@]}" -c bash -c rainlang-prelude)

nix develop -i "${keep[@]}" -c bash -c '(cd lib/rain.interpreter/lib/rain.metadata && rainix-rs-prelude)'

nix develop -i "${keep[@]}" -c rainix-sol-prelude
nix develop -i "${keep[@]}" -c rainix-rs-prelude
nix develop -i "${keep[@]}" -c raindex-prelude

nix develop -i "${keep[@]}" -c forge build

set +x

export LANG=en_US.UTF-8
export LC_ALL=en_US.UTF-8

printf "\033[0;32m"
printf "╔════════════════════════════════════════════════════════════════════════╗\n"
printf "║                          Base Setup Complete!                          ║\n"
printf "╚════════════════════════════════════════════════════════════════════════╝\n"
printf "\033[0m"

set -x
