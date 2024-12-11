#!/bin/bash

set -euo pipefail
current_path="$(realpath $0)"
current_dir="$(dirname $current_path)"

function package() {
	docker build -t sky/db-migrate:latest \
		-f "$current_dir/db/migrate.dockerfile" \
		"$current_dir/db"
}

function db_migrate() {
	docker run -v ./db/migrations:/migrations --network host migrate/migrate -path=/migrations/ -database "mysql://root:abc123456@tcp(localhost:3306)/sky?query" $@
}

function help() {
	echo "Usage: $(basename "$0") [OPTIONS]

Commands:
  package        Package docker images
  db_migrate  	 Run db migrate
  help           Show help
"
}

if [[ $1 =~ ^(package|db_migrate|help)$ ]]; then
	"$@"
else
	help
	exit 1
fi
