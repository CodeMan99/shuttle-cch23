#!/usr/bin/env bash

set -e

CCH23_PORT="${1:?port argument is required}"
NAMESPACE="cch23-codeman99"
LOGIN_EMAIL="codemister99@yahoo.com"
LOGIN_PASSWORD="cch23"
SERVER_JSON_TMP="$(mktemp server-XXX.json)"

# CONNECTION_STRING="postgres://postgres:postgres@localhost:${CCH23_PORT}/postgres"
cat << EOF >> "$SERVER_JSON_TMP"
{
    "Servers": {
        "1": {
            "Name": "$NAMESPACE",
            "Host": "localhost",
            "Username": "postgres",
            "Group": "Servers",
            "Port": $CCH23_PORT,
            "MaintenanceDB": "postgres",
            "SSLMode": "prefer"
        }
    }
}
EOF

docker build --pull --no-cache -f - -t "${NAMESPACE}/pgadmin4:latest" . << EOF
FROM dpage/pgadmin4:latest

COPY --chown=5050:5050 $SERVER_JSON_TMP /pgadmin4/servers.json
EOF

rm "$SERVER_JSON_TMP"

docker run --network host --name pgadmin \
    -e PGADMIN_CONFIG_LOGIN_BANNER="${NAMESPACE@Q}" \
    -e PGADMIN_DEFAULT_EMAIL="$LOGIN_EMAIL" \
    -e PGADMIN_DEFAULT_PASSWORD="$LOGIN_PASSWORD" \
    -e PGADMIN_LISTEN_PORT=5050 \
    -v pgadmin-data:/var/lib/pgadmin \
    -d "${NAMESPACE}/pgadmin4:latest"
