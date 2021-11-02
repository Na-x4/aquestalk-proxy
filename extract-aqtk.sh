#!/bin/sh
set -eux
cd "$(dirname "$0")"

if [ -e aquestalk ]; then
  exit 1
fi

TMPDIR=$(mktemp -d)

unzip -d "$TMPDIR" aqtk_mv_20090609.zip
mv "$TMPDIR"/AquesTalk_mv/bin ./aquestalk
find ./aquestalk/ -mindepth 1 -maxdepth 1 -type d -exec cp "$TMPDIR"/AquesTalk_mv/AqLicense.txt {} \;
rm -rf "$TMPDIR"
