#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
DATA_DIR="${SCRIPT_DIR}/../data"
mkdir -p "${DATA_DIR}"

TARGET_FILE="${DATA_DIR}/cambodia-latest.osm.pbf"
DOWNLOAD_URL="https://download.geofabrik.de/asia/cambodia-latest.osm.pbf"

echo "Downloading Cambodia OSM extract from Geofabrik..."
echo "Source: ${DOWNLOAD_URL}"
echo "Target: ${TARGET_FILE}"

curl -L -o "${TARGET_FILE}" "${DOWNLOAD_URL}"

echo "Download completed successfully!"
echo "Run: cargo run --release -- --data data/cambodia-latest.osm.pbf"
