#!/bin/bash
set -eu -o pipefail

cargo metadata --format-version=1 --no-deps |
  jq -r '.packages[] | select(.name == "aquestalk-proxyd") | .version'
