#!/usr/bin/env node
// seshport bin shim — execs the platform binary downloaded by install.js
const path = require("path");
const { spawnSync } = require("child_process");
const bin = path.join(__dirname, process.platform === "win32" ? "seshport.exe" : "seshport");
const r = spawnSync(bin, process.argv.slice(2), { stdio: "inherit" });
if (r.error) {
  console.error("seshport binary missing — reinstall (npm i -g seshport) or: cargo install seshport");
  process.exit(1);
}
process.exit(r.status ?? 0);
