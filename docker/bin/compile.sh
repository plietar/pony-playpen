#!/bin/sh

set -o errexit

ponyc --version

DIR="$(mktemp -d)"
mkdir "$DIR/main"
cd "$DIR/main"
cat > main.pony

ponyc --debug "$@"
printf '\377' # 255 in octal

[ -f main.ll ] && cat main.ll
[ -f main.s ] && cat main.s
