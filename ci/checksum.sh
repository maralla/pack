#!/usr/bin/env bash

set -ex

BREW_FILE=$1
VERSION=$2
MAC=$3
LINUX=$4


mac_checksum=$(shasum -a 256 $MAC | cut -f1 -d' ')
linux_checksum=$(shasum -a 256 $LINUX | cut -f1 -d' ')

# Substitude version
sed -i -b'' "s/\(version \)'[^']*'/\1'$VERSION'/" $BREW_FILE

# Substitude checksum
sed -i -b'' "s/\(sha256 \"\)[^\"]*\(\" # mac\)/\1$mac_checksum\2/" $BREW_FILE
sed -i -b'' "s/\(sha256 \"\)[^\"]*\(\" # linux\)/\1$linux_checksum\2/" $BREW_FILE
