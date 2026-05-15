#!/bin/bash

# Required parameters:
# @raycast.schemaVersion 1
# @raycast.title Paste with MARVIN
# @raycast.mode silent
# @raycast.packageName MARVIN

# Optional parameters:
# @raycast.icon ⌨️

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
pbpaste | "${SCRIPT_DIR}/../target/release/marvin-cli" -
