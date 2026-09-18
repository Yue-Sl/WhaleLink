#!/usr/bin/env bash
# Runs only on a protected self-hosted runner. Network inputs are supplied by
# GitHub Actions Variables/Secrets; never place production or test values here.
set -euo pipefail

fail() {
  printf '%s\n' "FAIL: $1" >&2
  exit 1
}

required() {
  local name="$1"
  [[ -n "${!name:-}" ]] || fail "missing required CI configuration: ${name}"
}

required EASYTIER_TEST_RELAY
required EASYTIER_TEST_NETWORK
required EASYTIER_TEST_SECRET
required EASYTIER_TEST_PEER

# Keep CI values as simple TOML scalars and prevent configuration injection.
[[ "$EASYTIER_TEST_RELAY" =~ ^tcp://[[:alnum:].-]+:[0-9]{1,5}$ ]] || fail 'invalid relay URL configuration'
[[ "$EASYTIER_TEST_NETWORK" =~ ^[[:alnum:]_.-]{1,128}$ ]] || fail 'invalid network name configuration'
[[ "$EASYTIER_TEST_SECRET" =~ ^[[:alnum:]_-]{16,256}$ ]] || fail 'invalid network secret configuration'
[[ "$EASYTIER_TEST_PEER" =~ ^([0-9]{1,3}\.){3}[0-9]{1,3}$ ]] || fail 'invalid peer IPv4 configuration'

expected_version="${EASYTIER_EXPECTED_VERSION:-2.6.4}"
requested_core="${EASYTIER_CORE_PATH:-easytier-core}"
if [[ "$requested_core" == */* ]]; then
  [[ -x "$requested_core" ]] || fail 'configured EasyTier executable is not executable'
  core="$requested_core"
else
  core="$(command -v "$requested_core" || true)"
  [[ -n "$core" ]] || fail 'EasyTier executable was not found on PATH'
fi

version="$($core --version 2>&1 || true)"
[[ "$version" == *"$expected_version"* ]] || fail 'installed EasyTier version does not match the locked CI version'
[[ -c /dev/net/tun ]] || fail '/dev/net/tun is unavailable'
sudo -n true || fail 'runner requires non-interactive sudo for the TUN smoke test'
command -v ip >/dev/null || fail 'iproute2 is unavailable'
command -v ping >/dev/null || fail 'ping is unavailable'

tmpdir="$(mktemp -d)"
config="$tmpdir/easytier-ci.toml"
runtime_log="$tmpdir/easytier-ci.log"
node_pid=''

cleanup() {
  local status=$?
  if [[ -n "$node_pid" ]] && sudo -n kill -0 "$node_pid" 2>/dev/null; then
    sudo -n kill "$node_pid" 2>/dev/null || true
  fi
  rm -rf -- "$tmpdir"
  trap - EXIT
  exit "$status"
}
trap cleanup EXIT INT TERM

umask 077
cat >"$config" <<EOF
network_name = "$EASYTIER_TEST_NETWORK"
network_secret = "$EASYTIER_TEST_SECRET"
dhcp = true
peers = ["$EASYTIER_TEST_RELAY"]
disable_upnp = true
console_log_level = "warn"
EOF

# The secret stays in a mode-0600 temporary file rather than appearing in a
# process list. Do not print the runtime log: it may contain network metadata.
node_pid="$(sudo -n sh -c '
  "$1" --config-file "$2" >"$3" 2>&1 &
  printf "%s" "$!"
' sh "$core" "$config" "$runtime_log")"
[[ "$node_pid" =~ ^[0-9]+$ ]] || fail 'EasyTier did not return a valid process identifier'

deadline=$((SECONDS + 120))
while (( SECONDS < deadline )); do
  if ! sudo -n kill -0 "$node_pid" 2>/dev/null; then
    fail 'EasyTier stopped before the TUN peer became reachable'
  fi

  route="$(ip route get "$EASYTIER_TEST_PEER" 2>/dev/null || true)"
  interface="$(sed -n 's/.* dev \([^ ]*\).*/\1/p' <<<"$route" | head -n 1)"
  if [[ -n "$interface" ]] \
    && [[ -e "/sys/class/net/$interface/tun_flags" ]] \
    && ping -n -c 3 -W 2 "$EASYTIER_TEST_PEER" >/dev/null 2>&1; then
    printf '%s\n' 'PASS: locked EasyTier created a TUN route and reached the protected remote test peer.'
    exit 0
  fi
  sleep 2
done

fail 'timed out waiting for a TUN route and ICMP reachability to the protected remote test peer'
