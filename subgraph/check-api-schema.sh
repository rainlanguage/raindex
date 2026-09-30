#!/usr/bin/env bash
set -euo pipefail

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
snapshot=crates/subgraph/schema/raindex.graphql
generated=raindex.graphql.generated
subgraph_name=rain/raindex

# Deployed in place, from the manifest as committed. Nothing here passes
# `graph build --network`: subgraph.yaml already carries its network, address
# and startBlock, and `--network` is what would write a networks.json entry
# back into it.
cd "$root/subgraph"
npm ci
graph build
graph create --node http://localhost:8020/ "$subgraph_name"
graph deploy \
  --node http://localhost:8020/ \
  --ipfs http://localhost:5001 \
  --version-label ci \
  "$subgraph_name"

# graph-node derives the API schema at deploy time but refuses queries until
# the deployment has ingested a block, so print-api-schema.js retries. Anvil's
# block 0 is the whole of what has to be ingested.
node ./print-api-schema.js "http://localhost:8000/subgraphs/name/$subgraph_name" \
  > "$root/$generated"

cd "$root"
if ! diff -u "$snapshot" "$generated"; then
  echo "$snapshot is not the API schema graph-node derives from subgraph/schema.graphql." >&2
  echo "Copy $generated (uploaded by this job as an artifact) over it." >&2
  exit 1
fi
