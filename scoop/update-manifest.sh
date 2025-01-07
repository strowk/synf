#!/bin/bash

version=$(cargo metadata --format-version=1 --no-deps | jq '.packages[0].version' -r)
cat << EOF > scoop/synf.json
{
    "version":  "${version}",
    "license":  "MIT",
    "extract_dir":  "target/x86_64-pc-windows-gnu/release",
    "url":  "https://github.com/strowk/synf/releases/download/v${version}/synf-x86_64-pc-windows-gnu.tar.gz",
    "homepage":  "https://github.com/strowk/synf",
    "bin":  "synf.exe"
}
EOF
