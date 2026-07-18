// seshport — by Harshil-Jani
// Hands a coding-agent session off to another agent (Claude Code <-> Codex, more to come)
// so the target tool can `resume` the conversation and keep going.
//
// Architecture: every tool implements the small `Tool` trait (sniff / import / export)
// against a neutral `Transcript`, so adding an integration never touches the others.
// Cross-tool resume can't replay provider-specific API state (tool-call ids, encrypted
// reasoning), so tool calls/results are flattened into readable text — the resumed agent
// gets the full story as context and continues from there.

use chrono::Utc;
use serde_json::{json, Value};
use std::fs;
use std::path::{Path, PathBuf};

const ATTRIBUTION: &str = "seshport by Harshil-Jani";
const TOOL_TEXT_CAP: usize = 10_000; // ponytail: flat cap per tool result; tune if resumes blow context

// ---------- neutral transcript ----------

#[derive(Clone, Copy, PartialEq)]
enum Role {
    User,
    Assistant,
}

struct Msg {
    role: Role,
    text: String,
    ts: String,
}

struct Transcript {
    source_tool: &'static str,
    source_id: String,
    cwd: String,
    msgs: Vec<Msg>,
}

// One integration = one impl of this. Import parses the tool's on-disk transcript into
// the neutral form; export writes the neutral form somewhere the tool's `resume` finds it.
trait Tool {
    fn name(&self) -> &'static str;
    fn root(&self) -> PathBuf;
    fn sniff(&self, lines: &[Value]) -> bool;
    fn import(&self, path: &Path) -> Result<Transcript, String>;
    /// Returns (output path, resume command).
    fn export(&self, t: &Transcript) -> Result<(PathBuf, String), String>;
}

fn tools() -> Vec<Box<dyn Tool>> {
    vec![Box::new(ClaudeCode), Box::new(Codex)]
}

// ---------- cli ----------

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let (src, target_name) = match &args[..] {
        [s] => (s.clone(), None),
        [s, t] => (s.clone(), Some(t.clone())),
        _ => {
            let names: Vec<_> = tools().iter().map(|t| t.name().to_string()).collect();
            eprintln!(
                "{ATTRIBUTION}\n\n\
                 Ports a session to another coding agent (direction is auto-detected).\n\n\
                 Usage:\n  \
                 seshport <session-id>          find it in any tool, convert to the other\n  \
                 seshport <path.jsonl>          detect format from content, convert to the other\n  \
                 seshport <tool>                newest session of that tool -> the other\n  \
                 seshport <session> <tool>      explicit target tool\n\n\
                 Tools: {}",
                names.join(", ")
            );
            std::process::exit(2);
        }
    };
    if let Err(e) = run(&src, target_name.as_deref()) {
        eprintln!("error: {e}");
        std::process::exit(1);
    }
}

fn run(src: &str, target_name: Option<&str>) -> Result<(), String> {
    let tools = tools();
    let (source_idx, path) = locate(&tools, src)?;

    let target = match target_name {
        Some(name) => tools
            .iter()
            .find(|t| t.name() == name)
            .ok_or_else(|| format!("unknown tool '{name}'"))?,
        None => {
            let others: Vec<_> = tools.iter().enumerate().filter(|(i, _)| *i != source_idx).collect();
            match &others[..] {
                [(_, t)] => *t,
                _ => return Err(format!(
                    "several possible targets — say which: seshport {src} <{}>",
                    others.iter().map(|(_, t)| t.name()).collect::<Vec<_>>().join("|")
                )),
            }
        }
    };
    if target.name() == tools[source_idx].name() {
        return Err(format!("session is already a {} session", target.name()));
    }

    let mut t = tools[source_idx].import(&path)?;
    t.msgs.insert(0, Msg {
        role: Role::User,
        text: format!(
            "[{ATTRIBUTION}] This conversation was imported from {} session {}. \
             Tool calls appear flattened as text. Continue where it left off.",
            t.source_tool, t.source_id
        ),
        ts: now_iso(),
    });

    let (out, resume) = target.export(&t)?;
    println!("{}", out.display());
    println!("resume with:  {resume}");
    Ok(())
}

// Figure out which transcript the user means and which tool it belongs to.
fn locate(tools: &[Box<dyn Tool>], src: &str) -> Result<(usize, PathBuf), String> {
    if let Some(i) = tools.iter().position(|t| t.name() == src) {
        return newest(&tools[i].root()).map(|p| (i, p));
    }
    let p = PathBuf::from(src);
    if p.is_file() {
        let lines = read_lines(&p)?;
        let head = &lines[..lines.len().min(5)];
        return tools
            .iter()
            .position(|t| t.sniff(head))
            .map(|i| (i, p.clone()))
            .ok_or_else(|| format!("{}: not a transcript of any known tool", p.display()));
    }
    let mut hits: Vec<(usize, PathBuf)> = Vec::new();
    for (i, t) in tools.iter().enumerate() {
        let mut found = Vec::new();
        find_jsonl(&t.root(), src, &mut found);
        hits.extend(found.into_iter().map(|p| (i, p)));
    }
    match hits.len() {
        0 => Err(format!("no session matching '{src}' in any known tool")),
        1 => Ok(hits.remove(0)),
        n => Err(format!("ambiguous id '{src}': {n} matches — pass a full path")),
    }
}

// Newest session file by mtime — the filesystem already tracks "most recent" for free.
fn newest(root: &Path) -> Result<PathBuf, String> {
    let mut files = Vec::new();
    find_jsonl(root, "", &mut files);
    files
        .into_iter()
        .max_by_key(|p| fs::metadata(p).and_then(|m| m.modified()).ok())
        .ok_or_else(|| format!("no sessions under {}", root.display()))
}

// ---------- shared helpers ----------

fn home() -> PathBuf {
    PathBuf::from(std::env::var("HOME").expect("HOME not set"))
}

fn now_iso() -> String {
    Utc::now().format("%Y-%m-%dT%H:%M:%S%.3fZ").to_string()
}

fn find_jsonl(dir: &Path, needle: &str, out: &mut Vec<PathBuf>) {
    let Ok(entries) = fs::read_dir(dir) else { return };
    for e in entries.flatten() {
        let p = e.path();
        if p.is_dir() {
            find_jsonl(&p, needle, out);
        } else if p.extension().is_some_and(|x| x == "jsonl")
            && p.file_name().unwrap().to_string_lossy().contains(needle)
        {
            out.push(p);
        }
    }
}

fn read_lines(path: &Path) -> Result<Vec<Value>, String> {
    let text = fs::read_to_string(path).map_err(|e| format!("{}: {e}", path.display()))?;
    Ok(text.lines().filter_map(|l| serde_json::from_str(l).ok()).collect())
}

fn write_jsonl(path: &Path, lines: &[Value]) -> Result<(), String> {
    let body: String = lines.iter().map(|l| l.to_string() + "\n").collect();
    fs::write(path, body).map_err(|e| format!("{}: {e}", path.display()))
}

fn cap(mut s: String) -> String {
    if s.len() > TOOL_TEXT_CAP {
        s.truncate(TOOL_TEXT_CAP);
        s.push_str("\n[...truncated by seshport]");
    }
    s
}

// Pull plain text out of a content value that may be a string or a list of typed blocks.
fn text_of(v: &Value) -> String {
    match v {
        Value::String(s) => s.clone(),
        Value::Array(blocks) => blocks
            .iter()
            .filter_map(|b| b.get("text").and_then(Value::as_str))
            .collect::<Vec<_>>()
            .join("\n"),
        _ => String::new(),
    }
}

fn text_or_raw(v: &Value) -> String {
    v.as_str().map(str::to_string).unwrap_or_else(|| v.to_string())
}

// ---------- Claude Code ----------

struct ClaudeCode;

impl Tool for ClaudeCode {
    fn name(&self) -> &'static str {
        "claude"
    }

    fn root(&self) -> PathBuf {
        home().join(".claude/projects")
    }

    fn sniff(&self, lines: &[Value]) -> bool {
        lines.iter().any(|l| l.get("sessionId").is_some())
    }

    fn import(&self, path: &Path) -> Result<Transcript, String> {
        let lines = read_lines(path)?;
        let cwd = lines
            .iter()
            .find_map(|l| l.get("cwd").and_then(Value::as_str))
            .unwrap_or(".")
            .to_string();
        let source_id = path.file_stem().unwrap().to_string_lossy().to_string();

        let mut msgs = Vec::new();
        for line in &lines {
            let kind = line.get("type").and_then(Value::as_str).unwrap_or("");
            if kind != "user" && kind != "assistant" {
                continue;
            }
            if line.get("isSidechain").and_then(Value::as_bool).unwrap_or(false) {
                continue;
            }
            let content = &line["message"]["content"];
            let text = match content {
                Value::String(s) => s.clone(),
                Value::Array(blocks) => blocks
                    .iter()
                    .filter_map(claude_block_text)
                    .collect::<Vec<_>>()
                    .join("\n"),
                _ => String::new(),
            };
            if text.trim().is_empty() {
                continue;
            }
            msgs.push(Msg {
                role: if kind == "assistant" { Role::Assistant } else { Role::User },
                text,
                ts: line.get("timestamp").and_then(Value::as_str).unwrap_or("").to_string(),
            });
        }
        Ok(Transcript { source_tool: self.name(), source_id, cwd, msgs })
    }

    fn export(&self, t: &Transcript) -> Result<(PathBuf, String), String> {
        let session_id = uuid::Uuid::new_v4().to_string();
        let mut out = Vec::new();
        let mut parent: Option<String> = None;

        for (i, m) in t.msgs.iter().enumerate() {
            let uuid = uuid::Uuid::new_v4().to_string();
            let ts = if m.ts.is_empty() { now_iso() } else { m.ts.clone() };
            let (kind, message) = match m.role {
                Role::User => ("user", json!({"role": "user", "content": m.text})),
                Role::Assistant => ("assistant", json!({
                    "id": format!("msg_seshport_{i}"), "type": "message", "role": "assistant",
                    "model": "seshport-import", "content": [{"type": "text", "text": m.text}],
                })),
            };
            out.push(json!({
                "parentUuid": parent, "isSidechain": false, "userType": "external",
                "cwd": t.cwd, "sessionId": session_id, "version": env!("CARGO_PKG_VERSION"),
                "gitBranch": "", "type": kind, "message": message, "uuid": uuid, "timestamp": ts,
            }));
            parent = Some(uuid);
        }

        // Claude encodes the project cwd into a directory name: non-alphanumerics become '-'.
        let encoded: String = t.cwd
            .chars()
            .map(|c| if c.is_ascii_alphanumeric() { c } else { '-' })
            .collect();
        let dir = self.root().join(&encoded);
        fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
        let file = dir.join(format!("{session_id}.jsonl"));
        write_jsonl(&file, &out)?;
        Ok((file, format!("cd {} && claude --resume {session_id}", t.cwd)))
    }
}

fn claude_block_text(b: &Value) -> Option<String> {
    match b.get("type").and_then(Value::as_str)? {
        "text" => Some(b["text"].as_str()?.to_string()),
        "thinking" => None, // provider-internal, not part of the conversation
        "tool_use" => Some(format!(
            "[tool call: {}]\n{}",
            b["name"].as_str().unwrap_or("?"),
            cap(b["input"].to_string())
        )),
        "tool_result" => Some(format!("[tool result]\n{}", cap(text_of(&b["content"])))),
        "image" => Some("[image attachment]".into()),
        _ => None,
    }
}

// ---------- Codex ----------

struct Codex;

impl Tool for Codex {
    fn name(&self) -> &'static str {
        "codex"
    }

    fn root(&self) -> PathBuf {
        home().join(".codex/sessions")
    }

    fn sniff(&self, lines: &[Value]) -> bool {
        lines.iter().any(|l| l.get("payload").is_some())
    }

    fn import(&self, path: &Path) -> Result<Transcript, String> {
        let lines = read_lines(path)?;
        let meta = lines
            .iter()
            .find(|l| l["type"] == "session_meta")
            .ok_or("no session_meta line in codex rollout")?;
        let cwd = meta["payload"]["cwd"].as_str().unwrap_or(".").to_string();
        let source_id = meta["payload"]["session_id"].as_str().unwrap_or("?").to_string();

        let mut msgs = Vec::new();
        for line in &lines {
            if line["type"] != "response_item" {
                continue;
            }
            let Some((role, text)) = codex_payload_text(&line["payload"]) else { continue };
            if text.trim().is_empty() {
                continue;
            }
            msgs.push(Msg {
                role,
                text,
                ts: line.get("timestamp").and_then(Value::as_str).unwrap_or("").to_string(),
            });
        }
        Ok(Transcript { source_tool: self.name(), source_id, cwd, msgs })
    }

    fn export(&self, t: &Transcript) -> Result<(PathBuf, String), String> {
        let new_id = uuid::Uuid::new_v4().to_string();
        let now = Utc::now();
        let ts = now_iso();

        // ChatGPT-auth Codex rejects resumed sessions whose meta lacks the official
        // base_instructions/originator, so borrow them from the newest real rollout.
        // Attribution therefore lives in the injected first message, not in originator.
        let mut meta = json!({
            "session_id": new_id, "id": new_id, "timestamp": ts, "cwd": t.cwd,
            "originator": "codex-tui", "cli_version": env!("CARGO_PKG_VERSION"), "source": "cli",
        });
        if let Some(tpl) = self.meta_template() {
            for k in ["base_instructions", "model_provider", "originator", "cli_version", "source", "thread_source"] {
                if let Some(v) = tpl.get(k) {
                    meta[k] = v.clone();
                }
            }
        }

        let mut out = Vec::new();
        out.push(json!({"timestamp": ts, "type": "session_meta", "payload": meta}));
        for m in &t.msgs {
            let (role, ctype) = match m.role {
                Role::User => ("user", "input_text"),
                Role::Assistant => ("assistant", "output_text"),
            };
            let ts = if m.ts.is_empty() { ts.clone() } else { m.ts.clone() };
            out.push(json!({"timestamp": ts, "type": "response_item", "payload": {
                "type": "message", "role": role, "content": [{"type": ctype, "text": m.text}],
            }}));
        }

        let dir = self.root().join(now.format("%Y/%m/%d").to_string());
        fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
        let file = dir.join(format!("rollout-{}-{new_id}.jsonl", now.format("%Y-%m-%dT%H-%M-%S")));
        write_jsonl(&file, &out)?;
        Ok((file, format!("codex resume {new_id}")))
    }
}

impl Codex {
    // session_meta payload of the newest real codex rollout that carries base_instructions.
    fn meta_template(&self) -> Option<Value> {
        let mut files = Vec::new();
        find_jsonl(&self.root(), "rollout-", &mut files);
        files.sort(); // YYYY/MM/DD dirs + timestamped names sort chronologically
        for f in files.iter().rev() {
            let Ok(lines) = read_lines(f) else { continue };
            for line in lines {
                if line["type"] == "session_meta" && !line["payload"]["base_instructions"].is_null() {
                    return Some(line["payload"].clone());
                }
            }
        }
        None
    }
}

// Map a codex response_item payload to (role, flattened text). None = skip.
fn codex_payload_text(p: &Value) -> Option<(Role, String)> {
    let ptype = p.get("type").and_then(Value::as_str)?;
    match ptype {
        "message" => {
            let role = p["role"].as_str()?;
            let text = text_of(&p["content"]);
            // developer/system prompts and injected env context belong to codex, not the conversation
            if role == "developer" || role == "system" || text.starts_with("<environment_context>") {
                return None;
            }
            Some((if role == "assistant" { Role::Assistant } else { Role::User }, text))
        }
        "reasoning" => None, // encrypted, provider-internal
        t if t.ends_with("_call") => Some((
            Role::Assistant,
            format!(
                "[tool call: {}]\n{}",
                p["name"].as_str().unwrap_or(t),
                cap(p.get("input").or(p.get("arguments")).map(text_or_raw).unwrap_or_default())
            ),
        )),
        t if t.ends_with("_call_output") => {
            Some((Role::User, format!("[tool result]\n{}", cap(text_of(&p["output"])))))
        }
        _ => None,
    }
}
