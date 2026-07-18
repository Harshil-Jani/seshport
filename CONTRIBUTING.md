# Contributing: add your editor

seshport's whole design exists so that adding an agent/editor is one self-contained PR.
Every integration is one implementation of the `Tool` trait — five methods — plus a demo
fixture. Nothing else changes; you never touch another tool's code.

**Worked example: [PR #1 — Grok Build support](https://github.com/Harshil-Jani/seshport/pull/1)**
follows every step below. Copy its shape.

## The five methods

```rust
impl Tool for YourEditor {
    fn name(&self) -> &'static str;    // the CLI keyword: `seshport youreditor`
    fn root(&self) -> PathBuf;         // where sessions live on disk
    fn sniff(&self, lines) -> bool;    // "is this transcript mine?" (must be unambiguous)
    fn import(&self, path) -> Transcript;   // your format -> neutral Transcript
    fn export(&self, t) -> (path, resume);  // neutral Transcript -> your format + resume cmd
}
```

Register it in `tools()` — one line. Done.

## Step by step

1. **Find the session format.** Open-source tool? Read its persistence code (the Grok PR
   cites `xai-org/grok-build`'s storage module — file paths and all). Closed? Run a tiny
   session and inspect the files it writes.
2. **Implement `Tool`** in `src/main.rs`. Flatten tool calls/results to readable text
   (`[tool call: ...]`) — provider API state never survives a cross-tool port; the story does.
   Skip provider-internal items (thinking/reasoning, synthetic injections).
3. **Add a demo fixture** in `demo/` — a tiny synthetic session with a *recallable fact*
   planted in it (ours use codewords like `GUAVA-99`). Never use a real session.
4. **Wire it into the tests** — add your fixture to `fixtures_import` and
   `sniff_is_unambiguous` in `src/main.rs`. `cargo test` must pass; ambiguous sniffing
   breaks direction auto-detection for everyone.
5. **Prove a resume.** Best: port the fixture and run your tool's `resume` for real.
   If your tool needs auth you can't script, validate what you can offline (the Grok PR
   uses `grok sessions list` — the tool's own reader parsing our output) and say exactly
   what's verified and what isn't in the PR description. Honesty over ceremony.

## PR checklist

- [ ] `impl Tool` + one line in `tools()`
- [ ] `demo/` fixture (synthetic, with a planted recallable fact)
- [ ] both tests updated and passing
- [ ] a real or best-effort resume verification, stated plainly
- [ ] README: add your tool to the usage list and the works-with badge
- [ ] optional: a `/seshport` slash-command file under `commands/` if your tool supports them

Attribution stays: every ported session's first message credits the source session and
seshport. Please keep that intact.
