#!/bin/dash

set -o errexit

ponyc --version

DIR="$(mktemp -d)"
mkdir "$DIR/main"
cd "$DIR/main"
cat > main.pony

ponyc --debug --verbose=0 "$@"
printf '\377' # 255 in octal
exec ./main
