#!/bin/bash

# Set strict error handling
set -euxo pipefail

echo "Starting project setup..."

# Environment variables that need to be set (commented out as reference)
# export CI_DEPLOY_SEPOLIA_RPC_URL=""
# export CI_FORK_SEPOLIA_DEPLOYER_ADDRESS=""
# export CI_FORK_SEPOLIA_BLOCK_NUMBER=""
# export CI_DEPLOY_POLYGON_RPC_URL=""
# export CI_SEPOLIA_METABOARD_URL=""
# export RPC_URL_ETHEREUM_FORK=""
# export COMMIT_SHA=""

# Keep environment variables when using nix-develop
keep=(
  -k CI_DEPLOY_SEPOLIA_RPC_URL
  -k CI_FORK_SEPOLIA_DEPLOYER_ADDRESS
  -k CI_FORK_SEPOLIA_BLOCK_NUMBER
  -k CI_DEPLOY_POLYGON_RPC_URL
  -k CI_SEPOLIA_METABOARD_URL
  -k RPC_URL_ETHEREUM_FORK
  -k COMMIT_SHA
  -k PUBLIC_WALLETCONNECT_PROJECT_ID
)

echo "Preparing base setup..."
./prep-base.sh

rm -rf target || true

echo "Building packages..."
nix develop -i ${keep[@]} -c bash -c '(npm run build -w @rainlanguage/raindex)'

rm -rf target || true

nix develop -i ${keep[@]} -c bash -c '(npm run build -w @rainlanguage/ui-components && npm run build -w @rainlanguage/webapp)'

# Temporarily disable command echoing
set +x

export LANG=en_US.UTF-8
export LC_ALL=en_US.UTF-8

GREEN='\033[0;32m'
NC='\033[0m' # No Color

# Print the completion message
printf "\033[0;32m" # Set text to green
printf "╔════════════════════════════════════════════════════════════════════════╗\n"
printf "║                            Setup Complete!                             ║\n"
printf "╠════════════════════════════════════════════════════════════════════════╣\n"
printf "║                          How to run the apps:                          ║\n"
printf "║                                                                        ║\n"
printf "║  To run webapp:     cd packages/webapp && nix develop -c npm run dev   ║\n"
printf "╚════════════════════════════════════════════════════════════════════════╝\n"
printf "\033[0m" # Reset text color

# Re-enable command echoing
set -x
