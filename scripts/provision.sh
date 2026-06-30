#!/usr/bin/env bash
# Resilient provisioning for Quitrax tooling on a flaky/slow link.
# Downloads resume (-C -) and auto-retry on the connection resets we keep hitting.
set -u
cd "$(dirname "$0")/.."
mkdir -p .localbin

CIRCOM_URL="https://github.com/iden3/circom/releases/download/v2.2.3/circom-windows-amd64.exe"
STELLAR_URL="https://github.com/stellar/stellar-cli/releases/download/v27.0.0/stellar-cli-27.0.0-x86_64-pc-windows-msvc.tar.gz"

# resumable, retrying download
dl() {  # dl <url> <outfile>
  curl -L --ssl-no-revoke \
       -C - \
       --retry 100 --retry-all-errors --retry-delay 3 \
       --connect-timeout 30 \
       -o "$2" "$1"
}

echo ">> circom"
dl "$CIRCOM_URL" .localbin/circom.exe && echo "circom OK ($(stat -c%s .localbin/circom.exe) bytes)"

echo ">> stellar-cli"
dl "$STELLAR_URL" .localbin/stellar-cli.tar.gz && {
  tar -xzf .localbin/stellar-cli.tar.gz -C .localbin && echo "stellar extracted: $(ls .localbin/stellar.exe 2>/dev/null || ls .localbin/*.exe)"
}
