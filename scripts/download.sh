#!/bin/bash
set -eu

REPOSITORY=Na-x4/aquestalk-proxy
FILENAME=${2-aquestalk-proxy.zip}

if [ -n "${GITHUB_API_TOKEN-}" ]; then
  GITHUB_API_HEADERS=(
    -H "Authorization: token ${GITHUB_API_TOKEN-}"
  )
else
  GITHUB_API_HEADERS=()
fi

if [ -n "${1-}" ]; then
  version=tags/v${1-}
else
  version=latest
fi

release_url="https://api.github.com/repos/$REPOSITORY/releases/$version"
release=$(mktemp)

status_code=$(
  curl \
    "${GITHUB_API_HEADERS[@]}" \
    -H "Accept: application/vnd.github.v3.raw" \
    -o "${release}" \
    -w '%{http_code}\n' \
    "$release_url"
)
if [ "$status_code" != "200" ]; then
  jq '.message // .' <"${release}" >&2
  exit 1
fi
asset_url=$(jq -re .assets[0].url <"${release}")
rm "${release}"

curl \
  "${GITHUB_API_HEADERS[@]}" \
  -H "Accept: application/octet-stream" \
  -o "$FILENAME" \
  -L \
  "$asset_url"
