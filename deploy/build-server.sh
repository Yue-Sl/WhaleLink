#!/bin/sh
set -eu

# Run from the extracted WhaleLink source bundle on a Linux x86_64 build host.
# This script never reads deployment secrets; provide them only through the
# protected files referenced by the systemd unit or Docker Compose deployment.
cargo build --locked --release --package whalelink-server
install -Dm755 target/release/whalelink-server "${DESTDIR:-/usr/local}/bin/whalelink-server"
echo "PASS: whalelink-server installed; install deploy/whalelink-server.service next."
