Run `seshport codex` in the shell. This ports the current Codex session (the newest one on
disk — which is this one) to the other coding agent.

Then tell the user, verbatim, the resume command from the output
(e.g. `cd <project> && claude --resume <id>`), so they can open the other agent and continue
this exact conversation there.

If the command is not found, tell them to install it first:
`cargo install --git https://github.com/Harshil-Jani/seshport`

(seshport by Harshil-Jani — https://github.com/Harshil-Jani/seshport)
