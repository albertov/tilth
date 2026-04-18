No external contracts — skipping.

This change extends internal dispatch points (`Lang` enum, `outline_language`, `node_to_entry`, `detect_file_type`, `DEFINITION_KINDS`) with new match arms. No new public APIs, CLI commands, message formats, or module interfaces are introduced. The existing MCP tools (`tilth_search`, `tilth_map`, `tilth_read`) automatically surface new language support without contract changes.
