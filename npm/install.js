// seshport npm installer — downloads the prebuilt binary for this platform
// from the matching GitHub release. By Harshil-Jani.
const fs = require("fs");
const path = require("path");
const https = require("https");
const zlib = require("zlib");
const { version } = require("./package.json");

const TARGETS = {
  "darwin-arm64": "aarch64-apple-darwin",
  "darwin-x64": "x86_64-apple-darwin",
  "linux-x64": "x86_64-unknown-linux-musl",
  "linux-arm64": "aarch64-unknown-linux-musl",
  "win32-x64": "x86_64-pc-windows-msvc",
};

const key = `${process.platform}-${process.arch}`;
const target = TARGETS[key];
if (!target) {
  console.error(`seshport: no prebuilt binary for ${key}. Try: cargo install seshport`);
  process.exit(1);
}

const ext = process.platform === "win32" ? ".exe" : "";
const url = `https://github.com/Harshil-Jani/seshport/releases/download/v${version}/seshport-${target}${ext}.gz`;
const dest = path.join(__dirname, `seshport${ext}`);

function get(url, redirects = 0) {
  return new Promise((resolve, reject) => {
    if (redirects > 5) return reject(new Error("too many redirects"));
    https.get(url, (res) => {
      if (res.statusCode >= 300 && res.statusCode < 400 && res.headers.location) {
        res.resume();
        return resolve(get(res.headers.location, redirects + 1));
      }
      if (res.statusCode !== 200) return reject(new Error(`HTTP ${res.statusCode} for ${url}`));
      resolve(res);
    }).on("error", reject);
  });
}

get(url)
  .then((res) => new Promise((resolve, reject) => {
    const out = fs.createWriteStream(dest, { mode: 0o755 });
    res.pipe(zlib.createGunzip()).pipe(out).on("finish", resolve).on("error", reject);
  }))
  .then(() => console.log(`seshport ${version} installed (${target})`))
  .catch((e) => {
    console.error(`seshport: download failed: ${e.message}`);
    console.error("Fallback: cargo install seshport  (or: brew install harshil-jani/tap/seshport)");
    process.exit(1);
  });
