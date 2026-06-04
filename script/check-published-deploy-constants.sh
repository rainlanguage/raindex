#!/usr/bin/env bash
# Prints "OK" iff every version published to the soldeer registry for `raindex`
# has a full suite of pinned deploy constants in LibRaindexDeploy.sol: a
# DEPLOYED_ADDRESS and DEPLOYED_CODEHASH (suffixed with the version) for each of
# the six deployed contracts.
#
# Consumed by LibRaindexDeployTaggedConstants.t.sol via FFI. Output is one of:
#   OK                  - every published version has its full constant suite
#   MISSING: <names...>  - one or more expected constants are absent
#   SKIP: <reason>       - the registry could not be reached (nothing verified)
#
# Always exits 0 so the test sees the message rather than an ffi failure.

lib="src/lib/deploy/LibRaindexDeploy.sol"

versions=$(
  curl -fsS "https://api.soldeer.xyz/api/v1/revision?project_name=raindex" 2>/dev/null \
    | grep -oE '"version":"[0-9][0-9.]*"' | cut -d'"' -f4 | sort -u
)

if [ -z "$versions" ]; then
  printf 'SKIP: could not fetch published soldeer versions'
  exit 0
fi

# The six deployed contracts' constant prefixes.
prefixes="RAINDEX_DEPLOYED \
SUB_PARSER_DEPLOYED \
ROUTE_PROCESSOR_DEPLOYED \
GENERIC_POOL_ARB_ORDER_TAKER_DEPLOYED \
ROUTE_PROCESSOR_ARB_ORDER_TAKER_DEPLOYED \
GENERIC_POOL_FLASH_BORROWER_DEPLOYED"

missing=""
for v in $versions; do
  suffix=$(printf '%s' "$v" | tr . _)
  for p in $prefixes; do
    for kind in ADDRESS CODEHASH; do
      name="${p}_${kind}_${suffix}"
      grep -qE "constant ${name} =" "$lib" || missing="${missing} ${name}"
    done
  done
done

if [ -n "$missing" ]; then
  printf 'MISSING:%s' "$missing"
else
  printf 'OK'
fi
