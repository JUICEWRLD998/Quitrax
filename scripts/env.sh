# Source this in every shell: `source scripts/env.sh`
export CARGO_HTTP_CHECK_REVOKE=false
# Resilient cargo networking for the flaky link: HTTP/1.1 (no multiplexing),
# abort+retry a stalled transfer instead of hanging forever, more retries.
export CARGO_HTTP_MULTIPLEXING=false
export CARGO_HTTP_LOW_SPEED_LIMIT=1000      # bytes/s
export CARGO_HTTP_TIMEOUT=30                 # seconds before a stalled xfer aborts
export CARGO_NET_RETRY=10
export PATH="$HOME/.cargo/bin:/c/Users/fadhm/AppData/Roaming/npm:$(pwd)/.localbin:$PATH"
