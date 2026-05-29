#!/bin/bash

set -euxo pipefail

nix develop -c forge soldeer install
nix develop -c forge build
