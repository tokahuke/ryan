#! /usr/bin/env bash
set -e

echo
echo "    📦 Installer for Ryan downloaded"

export TARGET="x86_64-unknown-linux-gnu"
export LATEST_LINK=$(
    curl -ILs -o /dev/null -w %{url_effective} https://github.com/tokahuke/ryan/releases/latest
)
export LATEST=$(
    echo $LATEST_LINK | grep --only-matching --color=never "v[0-9.]*\$"
)

cd /usr/local/bin
curl -L -Ss "https://github.com/tokahuke/ryan/releases/download/$LATEST/ryan-$LATEST-$TARGET.tar.xz" \
    | tar -OJxf - ryan-$LATEST-$TARGET/ryan > ./ryan
chmod +x ryan


echo $'    '🎉 Ryan $(tput bold)$(tput setaf 2)$LATEST$(tput sgr0) installed on \
    $(tput bold)$TARGET$(tput sgr0)!
echo
