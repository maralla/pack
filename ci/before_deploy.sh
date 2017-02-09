#!/usr/bin/env bash

set -ex

mk_tarball() {
    td=$(mktemp -d)
    out_dir=$(pwd)
    name="${PROJECT_NAME}-${TRAVIS_TAG}-${TARGET}"

    cp target/$TARGET/release/pack "$td/"
    cp README.md "$td/"

    pushd $td
    tar czf "$out_dir/$name.tar.gz" *
    popd
    rm -r $td
}

cargo build --target $TARGET --release
mk_tarball
