#!/usr/bin/env sh

# Exits as soon as any line fails.
set -xeuo pipefail

cd src/sqlparser_py

export MATURIN_USERNAME=${NEXUS_USER}
export MATURIN_PASSWORD=${NEXUS_PASSWORD}
export MATURIN_REPOSITORY_URL=https://n3-nexus3.advai.net/nexus/repository/pypi-hosted/
maturin publish
