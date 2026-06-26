#!/bin/sh
set -eu

DB_DIR="${HIPPOCORE_DB_DIR:-/data}"
PORT="${HIPPOCORE_PORT:-8080}"
API_KEY="${HIPPOCORE_API_KEY:-change-me}"
ADMIN_USER="${HIPPOCORE_ADMIN_USER:-admin}"
ADMIN_PASSWORD="${HIPPOCORE_ADMIN_PASSWORD:-}"
ADMIN_PASSWORD_FILE="${HIPPOCORE_ADMIN_PASSWORD_FILE:-$DB_DIR/admin-password}"

mkdir -p "$DB_DIR"

generate_password() {
  tr -dc 'A-Za-z0-9' < /dev/urandom | head -c 24
}

if [ -z "$ADMIN_PASSWORD" ]; then
  if [ -s "$ADMIN_PASSWORD_FILE" ]; then
    ADMIN_PASSWORD="$(cat "$ADMIN_PASSWORD_FILE")"
    chmod 0644 "$ADMIN_PASSWORD_FILE" 2>/dev/null || true
    echo "hippocore admin: using existing generated password from $ADMIN_PASSWORD_FILE"
  else
    ADMIN_PASSWORD="$(generate_password)"
    umask 022
    printf '%s\n' "$ADMIN_PASSWORD" > "$ADMIN_PASSWORD_FILE"
    chmod 0644 "$ADMIN_PASSWORD_FILE" 2>/dev/null || true
    echo "hippocore admin: generated password for this instance"
    echo "hippocore admin: user=$ADMIN_USER password=$ADMIN_PASSWORD"
    echo "hippocore admin: password saved at $ADMIN_PASSWORD_FILE"
  fi
else
  echo "hippocore admin: using HIPPOCORE_ADMIN_PASSWORD from environment"
fi

if [ "$(id -u)" = "0" ]; then
  chown -R hippocore:hippocore "$DB_DIR"
  exec su -s /bin/sh -c "exec hippocore serve --db \"$DB_DIR\" --port \"$PORT\" --api-key \"$API_KEY\" --admin-user \"$ADMIN_USER\" --admin-password \"$ADMIN_PASSWORD\"" hippocore
fi

exec hippocore serve --db "$DB_DIR" --port "$PORT" --api-key "$API_KEY" --admin-user "$ADMIN_USER" --admin-password "$ADMIN_PASSWORD"
