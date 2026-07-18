---
description: Port this session to another coding agent, continue where you left off (seshport by Harshil-Jani)
allowed-tools: Bash(seshport:*)
---

Run `seshport claude $ARGUMENTS` with the Bash tool. This ports the current Claude Code
session (the newest one on disk — which is this one) to the other coding agent.

Then tell the user, verbatim, the resume command from the output (e.g. `codex resume <id>`),
so they can open the other agent and continue this exact conversation there.

If the command is not found, tell them to install it first:
`cargo install seshport`
