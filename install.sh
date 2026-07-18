#!/bin/sh
# seshport installer — prebuilt binary + /seshport slash commands for Claude Code and Codex.
# Usage:  curl -fsSL https://raw.githubusercontent.com/Harshil-Jani/seshport/main/install.sh | sh
set -e

case "$(uname -s)-$(uname -m)" in
  Darwin-arm64)  target="aarch64-apple-darwin" ;;
  Darwin-x86_64) target="x86_64-apple-darwin" ;;
  Linux-x86_64)  target="x86_64-unknown-linux-musl" ;;
  Linux-aarch64|Linux-arm64) target="aarch64-unknown-linux-musl" ;;
  *) target="" ;;
esac

bindir="$HOME/.local/bin"
mkdir -p "$bindir"

if [ -n "$target" ]; then
  echo "Installing seshport (prebuilt, $target)..."
  curl -fsSL "https://github.com/Harshil-Jani/seshport/releases/latest/download/seshport-$target.gz" \
    | gunzip > "$bindir/seshport"
  chmod +x "$bindir/seshport"
elif command -v cargo >/dev/null; then
  echo "No prebuilt binary for this platform — building with cargo..."
  cargo install seshport --quiet
  bindir="$HOME/.cargo/bin"
else
  echo "No prebuilt binary for $(uname -s)-$(uname -m) and no cargo. Install Rust: https://rustup.rs"
  exit 1
fi

echo "Installing /seshport slash commands..."
base="https://raw.githubusercontent.com/Harshil-Jani/seshport/main/commands"
mkdir -p "$HOME/.claude/commands" "$HOME/.codex/prompts"
curl -fsSL "$base/claude/seshport.md" -o "$HOME/.claude/commands/seshport.md"
curl -fsSL "$base/codex/seshport.md" -o "$HOME/.codex/prompts/seshport.md"

case ":$PATH:" in
  *":$bindir:"*) ;;
  *) echo "NOTE: add $bindir to your PATH" ;;
esac

echo
echo "Done. Type /seshport inside Claude Code or Codex to port your session across."
