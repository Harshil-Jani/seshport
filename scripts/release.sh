#!/bin/sh
# Cut a release: bumps every version, tags, pushes. CI does the rest.
# Usage: scripts/release.sh 0.3.1
set -e
v=$1
[ -n "$v" ] || { echo "usage: scripts/release.sh <version>"; exit 1; }
sed -i '' "s/^version = \".*\"/version = \"$v\"/" Cargo.toml
sed -i '' "s/\"version\": \".*\"/\"version\": \"$v\"/" npm/package.json
cargo build --release -q # refresh Cargo.lock
git add -A && git commit -m "release v$v" && git tag "v$v"
git push && git push --tags
echo "v$v tagged; release workflow will publish GH binaries, crates.io, npm, PyPI"
