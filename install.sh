#!/bin/sh
# seshport installer — binary + /seshport slash commands for Claude Code and Codex.
# Usage:  curl -fsSL https://raw.githubusercontent.com/Harshil-Jani/seshport/main/install.sh | sh
set -e

command -v cargo >/dev/null || { echo "cargo not found — install Rust first: https://rustup.rs"; exit 1; }

echo "Installing seshport binary..."
cargo install seshport --quiet

echo "Installing /seshport slash commands..."
base="https://raw.githubusercontent.com/Harshil-Jani/seshport/main/commands"
mkdir -p "$HOME/.claude/commands" "$HOME/.codex/prompts"
curl -fsSL "$base/claude/seshport.md" -o "$HOME/.claude/commands/seshport.md"
curl -fsSL "$base/codex/seshport.md" -o "$HOME/.codex/prompts/seshport.md"

echo
echo "Done. Type /seshport inside Claude Code or Codex to port your session across."
