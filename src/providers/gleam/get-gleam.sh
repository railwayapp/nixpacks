#!/bin/sh
gleam_version=$1
buildarch=$(arch)

file_url=https://github.com/gleam-lang/gleam/releases/download/v$gleam_version/gleam-v$gleam_version-$buildarch-unknown-linux-musl.tar.gz

wget -O /gleam.tar.gz $file_url
tar -xzf /gleam.tar.gz -C /usr/bin