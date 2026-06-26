#!/bin/sh
set -eu

DB_DIR="${HIPPOCORE_DB_DIR:-/data}"
PORT="${HIPPOCORE_PORT:-8080}"
API_KEY="${HIPPOCORE_API_KEY:-change-me}"

mkdir -p "$DB_DIR"

if [ "$(id -u)" = "0" ]; then
  chown -R hippocore:hippocore "$DB_DIR"
  exec su -s /bin/sh -c "exec hippocore serve --db \"$DB_DIR\" --port \"$PORT\" --api-key \"$API_KEY\"" hippocore
fi

exec hippocore serve --db "$DB_DIR" --port "$PORT" --api-key "$API_KEY"
