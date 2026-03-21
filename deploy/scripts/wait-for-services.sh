#!/usr/bin/env bash
# wait-for-services.sh
# Waits for all dev infrastructure services to become healthy before proceeding.
set -euo pipefail

TIMEOUT="${WAIT_TIMEOUT:-120}"
INTERVAL="${WAIT_INTERVAL:-2}"

log() { echo "[wait-for-services] $*"; }

wait_for() {
  local name="$1"
  local check_cmd="$2"
  local elapsed=0

  log "Waiting for $name..."
  while ! eval "$check_cmd" > /dev/null 2>&1; do
    if [ "$elapsed" -ge "$TIMEOUT" ]; then
      log "ERROR: $name did not become ready within ${TIMEOUT}s"
      return 1
    fi
    sleep "$INTERVAL"
    elapsed=$((elapsed + INTERVAL))
  done
  log "$name is ready (${elapsed}s)"
}

wait_for "PostgreSQL" "pg_isready -h localhost -p 5432 -U amr"
wait_for "Redis"      "redis-cli -h localhost -p 6379 ping"
wait_for "MinIO"      "curl -sf http://localhost:9000/minio/health/live"
wait_for "MQTT"       "mosquitto_sub -h localhost -p 1883 -t '\$SYS/#' -C 1 -W 3"

log "All services are ready."
