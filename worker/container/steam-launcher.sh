#!/bin/bash
# Steam launcher for CS2 farming container.
# Waits for session files to be injected, then launches Steam + CS2.

STEAM_CONFIG_DIR="/home/farmuser/.steam/steam/config"

echo "[steam-launcher] Waiting for session injection (.ready marker)..."

while [ ! -f "${STEAM_CONFIG_DIR}/.ready" ]; do
    sleep 1
done

rm -f "${STEAM_CONFIG_DIR}/.ready"
echo "[steam-launcher] Session detected, launching Steam..."

# Use disk-backed tmp to avoid OOM during Steam operations
export TMPDIR=/var/tmp

exec steam \
    -silent \
    -no-browser \
    -console \
    -applaunch 730 \
    -nosound \
    -novid \
    -windowed \
    -w 384 \
    -h 288 \
    +connect_lobby default
