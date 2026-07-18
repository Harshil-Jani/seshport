# seshport

**Port your session between coding agents.** Convert a Claude Code session into a Codex one
(or vice versa), then just `resume` and keep going — full conversation context included.

By [Harshil-Jani](https://github.com/Harshil-Jani) · MIT

## Demo

![seshport demo — porting sessions between Claude Code and Codex](docs/demo.gif)

A Codex session's haiku, recited by Claude Code after the port. A Claude Code session's
fizzbuzz, recalled by Codex. Real recording, synthetic sessions — nothing staged.

## Install

```bash
git clone https://github.com/Harshil-Jani/seshport && cd seshport
cargo install --path .
```

## How to use

Direction is auto-detected — a session always converts to the other tool.

```bash
seshport claude                # newest Claude Code session -> Codex
seshport codex                 # newest Codex session -> Claude Code
seshport <session-id>          # found in any tool's sessions, converted to the other
seshport <path.jsonl>          # format detected from file content
seshport <session> <tool>      # explicit target, for when there are 3+ tools
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
seshport demo/codex-session.jsonl    # -> claude --resume <id>
seshport demo/claude-session.jsonl   # -> codex resume <id>
```

## How it works

- User and assistant messages transfer as-is into a neutral `Transcript`.
- Tool calls/results are flattened to readable text (`[tool call: Bash] ...`) — provider-specific
  API state (tool-call ids, encrypted reasoning) can't be replayed cross-provider, but the
  resumed agent gets the full story as context.
- Thinking/reasoning blocks are dropped (provider-internal).
- Codex output borrows `base_instructions` from your newest real rollout — ChatGPT-auth Codex
  rejects sessions without the official instructions.
- Every import starts with an attribution message noting the source session and this tool.

## Adding a tool

Implement the `Tool` trait in `src/main.rs` (five methods: `name`, `root`, `sniff`, `import`,
`export`) and add it to `tools()`. Nothing else changes — the neutral `Transcript` keeps
integrations independent, so N tools cost 2·N converters instead of N².

## License

MIT © Harshil Jani
