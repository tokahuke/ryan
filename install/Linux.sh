#! /usr/bin/env bash
set -e

export TARGET="x86_64-unknown-linux-gnu"
export LATEST_LINK=$(
    curl -ILs -o /dev/null -w %{url_effective} https://github.com/tokahuke/ryan/releases/latest
)
export LATEST=$(
    echo $LATEST_LINK | grep --only-matching --color=never "v[0-9.]*\$"
)

cd /usr/local/bin
curl -L -Ss "https://github.com/tokahuke/ryan/releases/download/$LATEST/ryan-$LATEST-$TARGET.tar.xz" \
    | tar Oxf - ryan-$LATEST-$TARGET/ryan > ./ryan
chmod +x ryan

echo
echo $'    '🎉 Ryan $(tput bold)$(tput setaf 2)$LATEST$(tput sgr0) installed on \
    $(tput bold)$TARGET$(tput sgr0)!
echo