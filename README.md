# seshport

Port your session between coding agents. Convert a **Claude Code** session into a
**Codex** one (or vice versa), then just `resume` and keep going.

By [Harshil-Jani](https://github.com/Harshil-Jani).

## Install

```bash
cargo install --path .
```

## Usage

Direction is auto-detected — a session always converts to the other tool.

```bash
seshport <session-id>          # found in any tool's sessions, converted to the other
seshport <path.jsonl>          # format detected from file content
seshport claude                # newest Claude Code session -> Codex
seshport codex                 # newest Codex session -> Claude Code
seshport <session> <tool>      # explicit target, for when there are 3+ tools
```

Each run prints the output path and the exact resume command
(`codex resume <id>` or `cd <project> && claude --resume <id>`).

## Demo

Synthetic transcripts live in [`demo/`](demo/) — try it without touching your real sessions:

```bash
seshport demo/claude-session.jsonl   # -> codex resume <id>
seshport demo/codex-session.jsonl    # -> cd ... && claude --resume <id>
```

Verified end-to-end: Claude Code resumes the ported Codex session and recites its haiku;
Codex resumes the ported Claude session and recalls the fizzbuzz it wrote.

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
