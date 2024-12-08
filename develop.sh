#!/bin/bash

set -euo pipefail
current_path="$(realpath $0)"
current_dir="$(dirname $current_path)"

function format() {
	# Format proto files
	find ./ -iname *.proto | xargs clang-format -style=Google -i
}

function help() {
	echo "Usage: $(basename "$0") [OPTIONS]

Commands:
  format         Format source files
  help           Show help
"
}

if [[ $1 =~ ^(format|help)$ ]]; then
	"$@"
else
	help
	exit 1
fi
