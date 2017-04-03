#!/bin/dash

set -o errexit

if [ "$RUST_PLAYPEN_ENV" != "irc" ]; then
    ponyc --version
fi

DIR="$(mktemp -d)"
mkdir "$DIR/main"
cd "$DIR/main"
cat > main.pony

ponyc "$@"
printf '\377' # 255 in octal

[ -f main.ll ] && cat main.ll
[ -f main.s ] && cat main.s
