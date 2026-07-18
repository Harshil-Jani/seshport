# seshport

[![crates.io](https://img.shields.io/crates/v/seshport?color=4a9eed)](https://crates.io/crates/seshport)
[![License: MIT](https://img.shields.io/badge/license-MIT-22c55e)](LICENSE)
[![Built with Rust](https://img.shields.io/badge/built%20with-Rust-f59e0b?logo=rust)](https://www.rust-lang.org)
[![Works with](https://img.shields.io/badge/works%20with-Claude%20Code%20%C2%B7%20Codex%20%C2%B7%20Grok%20Build-8b5cf6)](#how-to-use)
[![GitHub stars](https://img.shields.io/github/stars/Harshil-Jani/seshport?style=social)](https://github.com/Harshil-Jani/seshport/stargazers)

**Port your session between coding agents.** Type `/seshport` inside Claude Code or Codex,
open the other agent, and `resume` the exact same conversation — full context included.

By [Harshil-Jani](https://github.com/Harshil-Jani) · MIT

## Demo

![seshport demo — porting sessions between Claude Code and Codex](docs/demo.gif)

A Codex session's haiku, recited by Claude Code after the port. A Claude Code session's
fizzbuzz, recalled by Codex. Real recording, synthetic sessions — nothing staged.

## Install

One line — installs the binary **and** the `/seshport` slash command for both agents:

```bash
curl -fsSL https://raw.githubusercontent.com/Harshil-Jani/seshport/main/install.sh | sh
```

(Or manually: `cargo install seshport`, then copy
[`commands/`](commands/) into `~/.claude/commands/` and `~/.codex/prompts/`.)

## The easy way: `/seshport`

Never leave your agent. Mid-conversation, just type:

```
/seshport
```

- Inside **Claude Code** → replies with `codex resume <id>`
- Inside **Codex** → replies with `cd <project> && claude --resume <id>`

Open the other agent, paste, and continue the exact same conversation. Verified: a codeword
planted in a Claude Code session was recalled by Codex after a `/seshport` round-trip.

## CLI usage

The source tool is always auto-detected; name the target (or omit it when only one
target is possible).

```bash
seshport <session-id> <tool>   # port any session to any tool (claude | codex | grok)
seshport <session-id>          # auto: found in any tool, only one possible target
seshport <path.jsonl> <tool>   # format detected from file content
seshport claude codex          # newest Claude Code session -> Codex
seshport grok claude           # newest Grok Build session -> Claude Code
```

Each run prints the output path and the exact resume command:

```
$ seshport codex
/Users/you/.claude/projects/-your-project/1b2c3d4e-....jsonl
resume with:  cd /your/project && claude --resume 1b2c3d4e-...
```

Want to try without touching your real sessions? The demo transcripts from the GIF are in
[`demo/`](demo/):

```bash
seshport demo/codex-session.jsonl claude          # -> claude --resume <id>
seshport demo/claude-session.jsonl grok           # -> grok --resume <id>
seshport demo/grok-session/chat_history.jsonl codex   # -> codex resume <id>
```

## Architecture

![seshport architecture — sessions flow through a neutral Transcript; each agent is one Tool impl](docs/architecture.svg)

Every agent is one implementation of the `Tool` trait against a neutral `Transcript` — so N
tools cost 2·N converters instead of N². The diagram is editable: open
[`docs/architecture.excalidraw`](docs/architecture.excalidraw) at [excalidraw.com](https://excalidraw.com).

## How it works

- User and assistant messages transfer as-is into a neutral `Transcript`.
- Tool calls/results are flattened to readable text (`[tool call: Bash] ...`) — provider-specific
  API state (tool-call ids, encrypted reasoning) can't be replayed cross-provider, but the
  resumed agent gets the full story as context.
- Thinking/reasoning blocks are dropped (provider-internal).
- Codex output borrows `base_instructions` from your newest real rollout — ChatGPT-auth Codex
  rejects sessions without the official instructions.
- Every import starts with an attribution message noting the source session and this tool.

## Add your editor — PRs welcome

Every integration is one `Tool` impl (five methods) plus a demo fixture — see
[CONTRIBUTING.md](CONTRIBUTING.md) for the step-by-step guide, and
[PR #1](https://github.com/Harshil-Jani/seshport/pull/1) (Grok Build) for a worked example
you can copy.

## License

MIT © Harshil Jani
