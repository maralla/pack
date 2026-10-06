#!/usr/bin/env bash

set -ex

REF=$1
TARGET=$2

if [[ "$REF" == refs/tags/* ]]; then
    VERSION="${REF##refs/tags/}"
elif [[ "$REF" == refs/heads/* ]]; then
    VERSION="${REF##refs/heads/}"
else
    VERSION="unknown"
fi

VERSION="${VERSION//\//-}"

td=$(mktemp -d)
out_dir=$(pwd)
name="pack-${VERSION}-${TARGET}"

cp "target/${TARGET}/release/pack" "$td/"
cp README.md "$td/"
cp LICENSE "$td/"
cp -r contrib "$td/"

pushd "$td"
tar czf "$out_dir/$name.tar.gz" *
popd
rm -r "$td"

echo "name=$name.tar.gz" >> "$GITHUB_OUTPUT"
