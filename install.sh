#!/bin/bash

THESYSTEMIS="unknown-linux-gnu"
POSTFIX=""

if [[ "$OSTYPE" == "linux-gnu"* ]]; then
    THESYSTEMIS="unknown-linux-gnu"
elif [[ "$OSTYPE" == "darwin"* ]]; then
    THESYSTEMIS="apple-darwin"
elif [[ "$OSTYPE" == "cygwin" ]]; then
    THESYSTEMIS="pc-windows-gnu"
elif [[ "$OSTYPE" == "msys" ]]; then
    THESYSTEMIS="pc-windows-gnu"
elif [[ "$OSTYPE" == "win32" ]]; then
    THESYSTEMIS="pc-windows-gnu"
fi

if [[ "$THESYSTEMIS" == "unknown-linux-gnu" ]]; then
    libc=$(ldd /bin/ls | grep 'musl' | head -1 | cut -d ' ' -f1)
    if ! [ -z $libc ]; then
        THESYSTEMIS="unknown-linux-musl"
    fi
fi

if [[ "$THESYSTEMIS" == "pc-windows-gnu" ]]; then
    POSTFIX=".exe"
fi

echo "The system is $THESYSTEMIS"
ARCH="$(uname -m)"
echo "architecture is $ARCH"

BUILD="${ARCH}-${THESYSTEMIS}"
DOWNLOAD_URL="$(curl https://api.github.com/repos/strowk/synf/releases/latest | grep browser_download_url | grep ${BUILD} | cut -d '"' -f 4 )"

if [[ -z "$DOWNLOAD_URL" ]]; then
    echo "No prebuilt binary found for $BUILD"
    echo "Check out existing builds in https://github.com/strowk/synf/releases/latest"
    echo "Or you could build from source"
    echo "Refer to README in https://github.com/strowk/synf#from-sources for more information"
    exit 1
fi

echo "Downloading from $DOWNLOAD_URL"
curl "$DOWNLOAD_URL" -Lo ./synf-archive.tgz
mkdir -p ./synf-install
tar -xzf ./synf-archive.tgz -C ./synf-install

INSTALL_SOURCE="./synf-install/target/${BUILD}/release/synf${POSTFIX}"
INSTALL_TARGET="/usr/local/bin/synf"

chmod +x "${INSTALL_SOURCE}"

SUDO_PREFIX="sudo"

if [[ "$THESYSTEMIS" == "pc-windows-gnu" ]]; then
    mkdir -p /usr/local/bin
    SUDO_PREFIX=""
fi

# if set environment variable NO_SUDO, then don't use sudo
if [[ "$NO_SUDO" == "1" ]]; then
    SUDO_PREFIX=""
fi

${SUDO_PREFIX} mv "${INSTALL_SOURCE}" "${INSTALL_TARGET}${POSTFIX}"

rm synf-install/ -r
rm synf-archive.tgz

echo "synf is installed, checking by running 'synf --version'"
synf --version