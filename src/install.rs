use std::fs;
use std::path::PathBuf;

use serde_json::{json, Value};

/// Supported MCP hosts.
const SUPPORTED_HOSTS: &[&str] = &["claude-code", "cursor", "windsurf", "claude-desktop"];

/// Write MCP config for the given host, preserving existing servers.
pub fn run(host: &str) -> Result<(), String> {
    let config_path = config_path_for(host)?;

    let mut config: Value = if config_path.exists() {
        let raw = fs::read_to_string(&config_path)
            .map_err(|e| format!("failed to read {}: {e}", config_path.display()))?;
        serde_json::from_str(&raw)
            .map_err(|e| format!("invalid JSON in {}: {e}", config_path.display()))?
    } else {
        json!({})
    };

    config
        .as_object_mut()
        .ok_or("config root is not a JSON object")?
        .entry("mcpServers")
        .or_insert(json!({}))
        .as_object_mut()
        .ok_or("mcpServers is not a JSON object")?
        .insert(
            "tilth".into(),
            json!({
                "command": "tilth",
                "args": ["--mcp"]
            }),
        );

    if let Some(parent) = config_path.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| format!("failed to create {}: {e}", parent.display()))?;
    }

    let out =
        serde_json::to_string_pretty(&config).expect("serde_json::Value is always serializable");
    fs::write(&config_path, out)
        .map_err(|e| format!("failed to write {}: {e}", config_path.display()))?;

    eprintln!("✓ tilth added to {}", config_path.display());
    Ok(())
}

fn config_path_for(host: &str) -> Result<PathBuf, String> {
    match host {
        "claude-code" => Ok(PathBuf::from(".mcp.json")),
        "cursor" => Ok(PathBuf::from(".cursor/mcp.json")),
        "windsurf" => Ok(PathBuf::from(".windsurf/mcp.json")),
        "claude-desktop" => claude_desktop_path(),
        _ => Err(format!(
            "unknown host: {host}. Supported: {}",
            SUPPORTED_HOSTS.join(", ")
        )),
    }
}

fn claude_desktop_path() -> Result<PathBuf, String> {
    #[cfg(target_os = "macos")]
    {
        let home = std::env::var("HOME").map_err(|_| "HOME not set")?;
        Ok(PathBuf::from(home)
            .join("Library/Application Support/Claude/claude_desktop_config.json"))
    }

    #[cfg(target_os = "windows")]
    {
        let appdata = std::env::var("APPDATA").map_err(|_| "APPDATA not set")?;
        Ok(PathBuf::from(appdata).join("Claude/claude_desktop_config.json"))
    }

    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    {
        Err("claude-desktop config path unknown on this OS".into())
    }
}
