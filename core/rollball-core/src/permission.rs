//! Permission declaration and validation types
//!
//! Permissions follow a string-based format inspired by Android:
//! - `"network:https://api.weather.com"` — Network access with URL scope
//! - `"filesystem:read:~/Documents"` — Filesystem read with path scope
//! - `"filesystem:write:~/workdir"` — Filesystem write with path scope
//! - `"memory:read"` / `"memory:write"` — Memory access
//! - `"intent:send:com.example.calendar"` — Intent send
//! - `"intent:receive:com.example.weather"` — Intent receive
//! - `"identity:read"` / `"identity:write"` — Identity access
//! - `"shell"` — Shell command execution
//! - `"wasm"` — WASM tool execution
//! - `"rag:query"` — RAG tool query permission (Phase 4 S4.6)

use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::collections::HashMap;

// ── Permission binary encoding helpers (S5.5) ────────────────────────────

/// Permission type tags for compact binary encoding.
/// Each variant gets a unique 1-byte tag for efficient IPC transport.
#[repr(u8)]
enum PermissionTag {
    Network = 0,
    FilesystemRead = 1,
    FilesystemWrite = 2,
    MemoryRead = 3,
    MemoryWrite = 4,
    IntentSend = 5,
    IntentReceive = 6,
    IdentityRead = 7,
    IdentityWrite = 8,
    Shell = 9,
    Wasm = 10,
    RagQuery = 11,
}

impl Permission {
    fn tag(&self) -> u8 {
        match self {
            Permission::Network(_) => PermissionTag::Network as u8,
            Permission::FilesystemRead(_) => PermissionTag::FilesystemRead as u8,
            Permission::FilesystemWrite(_) => PermissionTag::FilesystemWrite as u8,
            Permission::MemoryRead => PermissionTag::MemoryRead as u8,
            Permission::MemoryWrite => PermissionTag::MemoryWrite as u8,
            Permission::IntentSend(_) => PermissionTag::IntentSend as u8,
            Permission::IntentReceive(_) => PermissionTag::IntentReceive as u8,
            Permission::IdentityRead => PermissionTag::IdentityRead as u8,
            Permission::IdentityWrite => PermissionTag::IdentityWrite as u8,
            Permission::Shell => PermissionTag::Shell as u8,
            Permission::Wasm => PermissionTag::Wasm as u8,
            Permission::RagQuery(_) => PermissionTag::RagQuery as u8,
        }
    }
}

// ── S5.5: Compact binary encoding for Permission ─────────────────────────

impl Permission {
    /// Encode this permission into a compact binary format.
    /// Format: [1 byte tag] [optional: 4 bytes len + UTF-8 value]
    pub fn encode_binary(&self, buf: &mut Vec<u8>) {
        buf.push(self.tag());
        if let Some(val) = self.type_value() {
            let len = val.len() as u32;
            buf.extend_from_slice(&len.to_le_bytes());
            buf.extend_from_slice(val.as_bytes());
        } else {
            buf.extend_from_slice(&0u32.to_le_bytes());
        }
    }

    /// Decode a permission from compact binary format.
    pub fn decode_binary(data: &[u8]) -> Result<(Self, usize), String> {
        if data.is_empty() {
            return Err("empty data".to_string());
        }
        let tag = data[0];
        if data.len() < 5 {
            return Err("truncated data".to_string());
        }
        let val_len = u32::from_le_bytes([data[1], data[2], data[3], data[4]]) as usize;
        let value = if val_len > 0 {
            if data.len() < 5 + val_len {
                return Err("truncated value".to_string());
            }
            Some(String::from_utf8(data[5..5 + val_len].to_vec())
                .map_err(|e| e.to_string())?)
        } else {
            None
        };
        let consumed = 5 + val_len;
        Ok((Permission::from_tag_simple(tag, value)?, consumed))
    }

    fn from_tag_simple(tag: u8, value: Option<String>) -> Result<Self, String> {
        match tag {
            0 => Ok(Permission::Network(value)),
            1 => Ok(Permission::FilesystemRead(value)),
            2 => Ok(Permission::FilesystemWrite(value)),
            3 => Ok(Permission::MemoryRead),
            4 => Ok(Permission::MemoryWrite),
            5 => Ok(Permission::IntentSend(value)),
            6 => Ok(Permission::IntentReceive(value)),
            7 => Ok(Permission::IdentityRead),
            8 => Ok(Permission::IdentityWrite),
            9 => Ok(Permission::Shell),
            10 => Ok(Permission::Wasm),
            11 => Ok(Permission::RagQuery(value)),
            _ => Err(format!("Unknown permission tag: {tag}")),
        }
    }
}

/// Permission types that Agents can declare in manifest
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Permission {
    /// Network access with optional URL whitelist
    /// None = full network access granted; Some(url) = restricted to that URL
    /// e.g., "network:https://api.weather.com"
    Network(Option<String>),
    /// Filesystem read access with optional path restriction
    /// None = full filesystem read; Some(path) = restricted to that path
    /// e.g., "filesystem:read:~/Documents"
    FilesystemRead(Option<String>),
    /// Filesystem write access with optional path restriction
    /// None = full filesystem write; Some(path) = restricted to that path
    /// e.g., "filesystem:write:~/workdir"
    FilesystemWrite(Option<String>),
    /// Memory read access
    MemoryRead,
    /// Memory write access
    MemoryWrite,
    /// Intent send to specific agent
    /// e.g., "intent:send:com.example.calendar"
    IntentSend(Option<String>),
    /// Intent receive from specific agent
    /// e.g., "intent:receive:com.example.weather"
    IntentReceive(Option<String>),
    /// Identity read access (query user identity fields)
    /// e.g., "identity:read"
    IdentityRead,
    /// Identity write access (store/update user identity fields)
    /// e.g., "identity:write"
    IdentityWrite,
    /// Shell command execution
    Shell,
    /// WASM tool execution
    Wasm,
    /// RAG tool query permission
    /// Required for agents with `[[tools]] type = "rag"` declaration.
    /// The optional scope is the RAG endpoint URL pattern.
    /// e.g., "rag:query" or "rag:query:https://rag.corp.example.com"
    RagQuery(Option<String>),
}

/// Error type for permission string parsing.
///
/// S5.2: Provides detailed error information when a permission string
/// cannot be parsed, including the invalid input and expected format.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PermissionParseError {
    /// The invalid input string.
    pub input: String,
    /// Human-readable description of the error.
    pub reason: String,
}

impl std::fmt::Display for PermissionParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Invalid permission '{}': {}", self.input, self.reason)
    }
}

impl std::error::Error for PermissionParseError {}

impl PermissionParseError {
    /// Create a new parse error with the invalid input and reason.
    fn new(input: &str, reason: &str) -> Self {
        Self {
            input: input.to_string(),
            reason: reason.to_string(),
        }
    }

    /// Create an error for an unknown permission category.
    fn unknown_category(input: &str, category: &str) -> Self {
        Self::new(
            input,
            &format!(
                "unknown category '{}'. Expected one of: network, filesystem, memory, intent, identity, shell, wasm, rag",
                category
            ),
        )
    }

    /// Create an error for a missing component (e.g., no access type after category).
    fn missing_component(input: &str, expected: &str) -> Self {
        Self::new(input, &format!("missing '{}'. Expected format: {}", expected, Self::expected_format(input)))
    }

    /// Create an error for an invalid sub-component value.
    #[allow(dead_code)]
    fn invalid_value(input: &str, component: &str, valid: &str) -> Self {
        Self::new(input, &format!("invalid {} '{}'. Valid values: {}", component, input, valid))
    }

    /// Determine the expected format hint based on the input prefix.
    fn expected_format(input: &str) -> &'static str {
        if input.starts_with("filesystem:") {
            "filesystem:<read|write>:<path>"
        } else if input.starts_with("intent:") {
            "intent:<send|receive>:<target>"
        } else if input.starts_with("rag:") {
            "rag:query[:<url>]"
        } else {
            "<category>:<spec>"
        }
    }
}

impl Permission {
    /// Parse a permission string into a Permission enum.
    ///
    /// Returns `Result<Permission, PermissionParseError>` with detailed
    /// error information on failure.
    ///
    /// # Examples
    /// ```
    /// use rollball_core::permission::Permission;
    /// let p = Permission::parse("network:https://api.weather.com").unwrap();
    /// assert!(matches!(p, Permission::Network(Some(_))));
    ///
    /// let err = Permission::parse("invalid").unwrap_err();
    /// assert!(err.reason.contains("missing category delimiter"));
    ///
    /// let err = Permission::parse("foo:bar").unwrap_err();
    /// assert!(err.reason.contains("unknown category"));
    /// ```
    pub fn parse(s: &str) -> Result<Self, PermissionParseError> {
        // Handle simple single-word permissions first
        if s == "shell" {
            return Ok(Permission::Shell);
        }
        if s == "wasm" {
            return Ok(Permission::Wasm);
        }
        if s == "rag:query" {
            return Ok(Permission::RagQuery(None));
        }

        // Split on the first colon only to get the category
        let (category, rest) = s.split_once(':')
            .ok_or_else(|| PermissionParseError::new(s, "missing category delimiter ':'. Expected format: <category>:<spec> or 'shell'/'wasm'"))?;
        match category {
            "network" => Ok(Permission::Network(Some(rest.to_string()))),
            "filesystem" => {
                // Split rest on first colon: "read:~/Documents" or "write:~/workdir"
                let (access, path) = rest.split_once(':')
                    .ok_or_else(|| PermissionParseError::missing_component(s, "access:path"))?;
                let path = Some(path.to_string());
                match access {
                    "read" => Ok(Permission::FilesystemRead(path)),
                    "write" => Ok(Permission::FilesystemWrite(path)),
                    other => Err(PermissionParseError::new(s, &format!("invalid filesystem access '{}'. Expected 'read' or 'write'", other))),
                }
            }
            "memory" => match rest {
                "read" => Ok(Permission::MemoryRead),
                "write" => Ok(Permission::MemoryWrite),
                other => Err(PermissionParseError::new(s, &format!("invalid memory operation '{}'. Expected 'read' or 'write'", other))),
            },
            "intent" => {
                let (direction, target) = rest.split_once(':')
                    .ok_or_else(|| PermissionParseError::missing_component(s, "direction:target"))?;
                let target = Some(target.to_string());
                match direction {
                    "send" => Ok(Permission::IntentSend(target)),
                    "receive" => Ok(Permission::IntentReceive(target)),
                    other => Err(PermissionParseError::new(s, &format!("invalid intent direction '{}'. Expected 'send' or 'receive'", other))),
                }
            }
            "identity" => match rest {
                "read" => Ok(Permission::IdentityRead),
                "write" => Ok(Permission::IdentityWrite),
                other => Err(PermissionParseError::new(s, &format!("invalid identity operation '{}'. Expected 'read' or 'write'", other))),
            },
            "rag" => match rest {
                "query" => Ok(Permission::RagQuery(None)),
                query_part => {
                    // "rag:query:https://rag.corp.example.com"
                    if let Some(url) = query_part.strip_prefix("query:") {
                        Ok(Permission::RagQuery(Some(url.to_string())))
                    } else {
                        Err(PermissionParseError::new(s, &format!("invalid rag operation '{}'. Expected 'query' or 'query:<url>'", query_part)))
                    }
                }
            },
            other => Err(PermissionParseError::unknown_category(s, other)),
        }
    }

    /// Convert permission to string representation
    pub fn to_permission_string(&self) -> String {
        match self {
            Permission::Network(Some(url)) => format!("network:{url}"),
            Permission::Network(None) => "network".to_string(),
            Permission::FilesystemRead(Some(path)) => format!("filesystem:read:{path}"),
            Permission::FilesystemRead(None) => "filesystem:read".to_string(),
            Permission::FilesystemWrite(Some(path)) => format!("filesystem:write:{path}"),
            Permission::FilesystemWrite(None) => "filesystem:write".to_string(),
            Permission::MemoryRead => "memory:read".to_string(),
            Permission::MemoryWrite => "memory:write".to_string(),
            Permission::IntentSend(Some(target)) => format!("intent:send:{target}"),
            Permission::IntentSend(None) => "intent:send".to_string(),
            Permission::IntentReceive(Some(source)) => format!("intent:receive:{source}"),
            Permission::IntentReceive(None) => "intent:receive".to_string(),
            Permission::IdentityRead => "identity:read".to_string(),
            Permission::IdentityWrite => "identity:write".to_string(),
            Permission::Shell => "shell".to_string(),
            Permission::Wasm => "wasm".to_string(),
            Permission::RagQuery(None) => "rag:query".to_string(),
            Permission::RagQuery(Some(url)) => format!("rag:query:{url}"),
        }
    }

    /// Check if this permission matches (covers) a requested permission.
    /// A broader permission (e.g., `Network(None)`) matches a narrower one
    /// (e.g., `Network(Some("https://api.weather.com"))`).
    ///
    /// Broad→narrow semantics: `Network(None)` = "all network access",
    /// so it covers any `Network(Some(_))`. Conversely, `Network(Some(url))`
    /// only covers the exact same URL or `Network(None)` is required for broader.
    pub fn matches(&self, requested: &Permission) -> bool {
        match (self, requested) {
            // Same type: broader scope (None) matches narrower scope (Some)
            (Permission::Network(None), Permission::Network(_)) => true,
            (Permission::Network(a), Permission::Network(b)) => a == b,
            (Permission::FilesystemRead(None), Permission::FilesystemRead(_)) => true,
            (Permission::FilesystemRead(a), Permission::FilesystemRead(b)) => a == b,
            (Permission::FilesystemWrite(None), Permission::FilesystemWrite(_)) => true,
            (Permission::FilesystemWrite(a), Permission::FilesystemWrite(b)) => a == b,
            (Permission::MemoryRead, Permission::MemoryRead) => true,
            (Permission::MemoryWrite, Permission::MemoryWrite) => true,
            (Permission::IntentSend(None), Permission::IntentSend(_)) => true,
            (Permission::IntentSend(a), Permission::IntentSend(b)) => a == b,
            (Permission::IntentReceive(None), Permission::IntentReceive(_)) => true,
            (Permission::IntentReceive(a), Permission::IntentReceive(b)) => a == b,
            (Permission::IdentityRead, Permission::IdentityRead) => true,
            (Permission::IdentityWrite, Permission::IdentityWrite) => true,
            (Permission::Shell, Permission::Shell) => true,
            (Permission::Wasm, Permission::Wasm) => true,
            (Permission::RagQuery(None), Permission::RagQuery(_)) => true,
            (Permission::RagQuery(a), Permission::RagQuery(b)) => a == b,
            _ => false,
        }
    }

    /// Get the category of this permission (e.g., "network", "filesystem")
    pub fn category(&self) -> &str {
        match self {
            Permission::Network(_) => "network",
            Permission::FilesystemRead(_) | Permission::FilesystemWrite(_) => "filesystem",
            Permission::MemoryRead | Permission::MemoryWrite => "memory",
            Permission::IntentSend(_) | Permission::IntentReceive(_) => "intent",
            Permission::IdentityRead | Permission::IdentityWrite => "identity",
            Permission::Shell => "shell",
            Permission::Wasm => "wasm",
            Permission::RagQuery(_) => "rag",
        }
    }

    /// Get the type name for DB/serialization storage.
    /// Returns the PascalCase variant name (e.g., "Network", "FilesystemRead").
    pub fn type_name(&self) -> &str {
        match self {
            Permission::Network(_) => "Network",
            Permission::FilesystemRead(_) => "FilesystemRead",
            Permission::FilesystemWrite(_) => "FilesystemWrite",
            Permission::MemoryRead => "MemoryRead",
            Permission::MemoryWrite => "MemoryWrite",
            Permission::IntentSend(_) => "IntentSend",
            Permission::IntentReceive(_) => "IntentReceive",
            Permission::IdentityRead => "IdentityRead",
            Permission::IdentityWrite => "IdentityWrite",
            Permission::Shell => "Shell",
            Permission::Wasm => "Wasm",
            Permission::RagQuery(_) => "RagQuery",
        }
    }

    /// Get the scoped value for DB/serialization storage.
    /// Returns Some(value) for permissions with scope, None otherwise.
    pub fn type_value(&self) -> Option<&str> {
        match self {
            Permission::Network(v) => v.as_deref(),
            Permission::FilesystemRead(v) => v.as_deref(),
            Permission::FilesystemWrite(v) => v.as_deref(),
            Permission::IntentSend(v) => v.as_deref(),
            Permission::IntentReceive(v) => v.as_deref(),
            Permission::RagQuery(v) => v.as_deref(),
            Permission::MemoryRead
            | Permission::MemoryWrite
            | Permission::IdentityRead
            | Permission::IdentityWrite
            | Permission::Shell
            | Permission::Wasm => None,
        }
    }
}

// Custom TOML serialization using tagged enum format
impl Serialize for Permission {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        #[derive(Serialize)]
        struct PermissionRepr {
            #[serde(rename = "type")]
            perm_type: String,
            #[serde(skip_serializing_if = "Option::is_none")]
            value: Option<String>,
        }

        let repr = PermissionRepr {
            perm_type: self.type_name().to_string(),
            value: self.type_value().map(|s| s.to_string()),
        };
        repr.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for Permission {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        #[derive(Deserialize)]
        struct PermissionRepr {
            #[serde(rename = "type")]
            perm_type: String,
            #[serde(default)]
            value: Option<String>,
        }

        let repr = PermissionRepr::deserialize(deserializer)?;
        match repr.perm_type.as_str() {
            "Network" => Ok(Permission::Network(repr.value)),
            "FilesystemRead" => Ok(Permission::FilesystemRead(repr.value)),
            "FilesystemWrite" => Ok(Permission::FilesystemWrite(repr.value)),
            "MemoryRead" => Ok(Permission::MemoryRead),
            "MemoryWrite" => Ok(Permission::MemoryWrite),
            "IntentSend" => Ok(Permission::IntentSend(repr.value)),
            "IntentReceive" => Ok(Permission::IntentReceive(repr.value)),
            "IdentityRead" => Ok(Permission::IdentityRead),
            "IdentityWrite" => Ok(Permission::IdentityWrite),
            "Shell" => Ok(Permission::Shell),
            "Wasm" => Ok(Permission::Wasm),
            "RagQuery" => Ok(Permission::RagQuery(repr.value)),
            other => Err(serde::de::Error::custom(format!(
                "Unknown permission type: {other}"
            ))),
        }
    }
}

// ── Permission Grant & Policy (S1.1) ─────────────────────────────────────

/// Record of a permission being granted to an Agent.
///
/// Tracks who authorized what, when, and with what scope.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PermissionGrant {
    /// The Agent this grant belongs to
    pub agent_id: String,
    /// The permission that was granted
    pub permission: Permission,
    /// Who authorized this grant ("user", "system", or "auto")
    pub authorized_by: String,
    /// When the grant was created (Unix timestamp millis)
    pub granted_at: i64,
    /// Optional expiry time (Unix timestamp millis); None = permanent
    pub expires_at: Option<i64>,
    /// Scope constraint: e.g., a path or URL pattern further restricting the permission
    #[serde(default)]
    pub scope: Option<String>,
}

impl PermissionGrant {
    /// Create a new permanent permission grant
    pub fn new(agent_id: &str, permission: Permission, authorized_by: &str) -> Self {
        Self {
            agent_id: agent_id.to_string(),
            permission,
            authorized_by: authorized_by.to_string(),
            granted_at: chrono::Utc::now().timestamp_millis(),
            expires_at: None,
            scope: None,
        }
    }

    /// Create a grant with expiry
    pub fn with_expiry(agent_id: &str, permission: Permission, authorized_by: &str, expires_at: i64) -> Self {
        Self {
            expires_at: Some(expires_at),
            ..Self::new(agent_id, permission, authorized_by)
        }
    }

    /// Check if this grant has expired
    pub fn is_expired(&self) -> bool {
        match self.expires_at {
            Some(exp) => chrono::Utc::now().timestamp_millis() > exp,
            None => false,
        }
    }

    /// Check if this grant matches a requested permission
    ///
    /// A grant matches if: (1) it's not expired, and (2) the granted permission
    /// covers the requested permission (broad → narrow semantics).
    pub fn matches_request(&self, requested: &Permission) -> bool {
        !self.is_expired() && self.permission.matches(requested)
    }

    // ── S5.5: Compact binary serialization for IPC ─────────────────────

    /// Serialize this grant to compact binary bytes for IPC transport.
    /// Format: agent_id(str) + permission(binary) + authorized_by(str)
    ///          + granted_at(i64 LE) + expires_at(opt i64 LE) + scope(opt str)
    pub fn to_bincode(&self) -> Result<Vec<u8>, String> {
        let mut buf = Vec::new();
        // agent_id
        write_str(&mut buf, &self.agent_id);
        // permission
        self.permission.encode_binary(&mut buf);
        // authorized_by
        write_str(&mut buf, &self.authorized_by);
        // granted_at (i64 in little-endian)
        buf.extend_from_slice(&self.granted_at.to_le_bytes());
        // expires_at (Option<i64>): 1 byte present + 8 bytes value
        match self.expires_at {
            Some(v) => {
                buf.push(1u8);
                buf.extend_from_slice(&v.to_le_bytes());
            }
            None => buf.push(0u8),
        }
        // scope (Option<String>)
        match &self.scope {
            Some(s) => {
                buf.push(1u8);
                write_str(&mut buf, s);
            }
            None => buf.push(0u8),
        }
        Ok(buf)
    }

    /// Deserialize a grant from compact binary bytes (IPC transport).
    pub fn from_bincode(data: &[u8]) -> Result<Self, String> {
        let mut pos = 0;
        let agent_id = read_str(data, &mut pos)?;
        let (permission, consumed) = Permission::decode_binary(&data[pos..])?;
        pos += consumed;
        let authorized_by = read_str(data, &mut pos)?;
        if data.len() < pos + 8 {
            return Err("truncated grant".to_string());
        }
        let granted_at = i64::from_le_bytes([
            data[pos], data[pos+1], data[pos+2], data[pos+3],
            data[pos+4], data[pos+5], data[pos+6], data[pos+7],
        ]);
        pos += 8;
        if data.len() < pos + 1 {
            return Err("truncated expires_at".to_string());
        }
        let expires_at = if data[pos] == 1 {
            pos += 1;
            if data.len() < pos + 8 {
                return Err("truncated expires_at value".to_string());
            }
            let v = i64::from_le_bytes([
                data[pos], data[pos+1], data[pos+2], data[pos+3],
                data[pos+4], data[pos+5], data[pos+6], data[pos+7],
            ]);
            pos += 8;
            Some(v)
        } else {
            pos += 1;
            None
        };
        if data.len() < pos + 1 {
            return Err("truncated scope".to_string());
        }
        let scope = if data[pos] == 1 {
            pos += 1;
            Some(read_str(data, &mut pos)?)
        } else {
            pos += 1; // skip scope-present byte
            None
        };
        Ok(PermissionGrant {
            agent_id,
            permission,
            authorized_by,
            granted_at,
            expires_at,
            scope,
        })
    }

    /// Serialize this grant to JSON (backward-compatible format).
    pub fn to_json(&self) -> Result<Vec<u8>, String> {
        serde_json::to_vec(self).map_err(|e| e.to_string())
    }

    /// Deserialize a grant from JSON bytes (backward-compatible).
    pub fn from_json(data: &[u8]) -> Result<Self, String> {
        serde_json::from_slice(data).map_err(|e| e.to_string())
    }
}

/// Write a length-prefixed UTF-8 string to a buffer.
fn write_str(buf: &mut Vec<u8>, s: &str) {
    let len = s.len() as u32;
    buf.extend_from_slice(&len.to_le_bytes());
    buf.extend_from_slice(s.as_bytes());
}

/// Read a length-prefixed UTF-8 string from a buffer at position.
fn read_str(data: &[u8], pos: &mut usize) -> Result<String, String> {
    if data.len() < *pos + 4 {
        return Err("truncated string length".to_string());
    }
    let len = u32::from_le_bytes([data[*pos], data[*pos+1], data[*pos+2], data[*pos+3]]) as usize;
    *pos += 4;
    if data.len() < *pos + len {
        return Err("truncated string body".to_string());
    }
    let s = String::from_utf8(data[*pos..*pos + len].to_vec())
        .map_err(|e| e.to_string())?;
    *pos += len;
    Ok(s)
}

/// Policy for how a permission category should be handled by default.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PermissionPolicy {
    /// Auto-grant without asking the user (for low-risk permissions like memory:read)
    Allow,
    /// Auto-deny without asking the user
    Deny,
    /// Ask the user every time (for high-risk permissions like shell)
    AskAlways,
    /// Use the platform default policy
    Default,
}

impl PermissionPolicy {
    /// Get the default policy for a permission type
    pub fn for_permission(perm: &Permission) -> Self {
        match perm {
            // Low-risk: auto-grant
            Permission::MemoryRead => PermissionPolicy::Allow,
            Permission::IdentityRead => PermissionPolicy::Allow,
            Permission::IntentReceive(_) => PermissionPolicy::Allow,
            // High-risk: always ask
            Permission::Shell => PermissionPolicy::AskAlways,
            Permission::IdentityWrite => PermissionPolicy::AskAlways,
            Permission::Wasm => PermissionPolicy::AskAlways,
            // RAG: medium-risk, ask on first use
            Permission::RagQuery(_) => PermissionPolicy::Default,
            // Medium-risk: ask on first use
            Permission::Network(_) => PermissionPolicy::Default,
            Permission::FilesystemRead(_) => PermissionPolicy::Default,
            Permission::FilesystemWrite(_) => PermissionPolicy::Default,
            Permission::MemoryWrite => PermissionPolicy::Default,
            Permission::IntentSend(_) => PermissionPolicy::Default,
        }
    }

    /// Get the policy for a permission, optionally overridden by user config.
    /// When `config` provides a policy for this permission's category, that
    /// overrides the built-in default. When the configured policy is `Default`,
    /// falls back to the built-in default.
    pub fn for_permission_with_config(perm: &Permission, config: Option<&PermissionPolicyConfig>) -> Self {
        if let Some(cfg) = config {
            if let Some(policy) = cfg.get_policy(perm) {
                if policy != PermissionPolicy::Default {
                    return policy;
                }
            }
        }
        Self::for_permission(perm)
    }
}

/// Runtime-configurable permission policy overrides.
///
/// Stores per-category policy overrides loaded from config.
/// Categories are permission category names (e.g. "network", "filesystem", "shell").
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermissionPolicyConfig {
    /// Per-category policy overrides.
    /// Key is the permission category name (e.g. "network", "shell").
    /// Value is the policy to apply for that category.
    #[serde(default)]
    pub policies: HashMap<String, PermissionPolicy>,
}

impl PermissionPolicyConfig {
    /// Create an empty policy config (all defaults).
    pub fn new() -> Self {
        Self {
            policies: HashMap::new(),
        }
    }

    /// Get the configured policy for a permission, if any.
    /// Returns None if no override is configured for this permission's category.
    pub fn get_policy(&self, perm: &Permission) -> Option<PermissionPolicy> {
        self.policies.get(perm.category()).copied()
    }

    /// Set a policy override for a category.
    pub fn set_policy(&mut self, category: &str, policy: PermissionPolicy) {
        if policy == PermissionPolicy::Default {
            self.policies.remove(category);
        } else {
            self.policies.insert(category.to_string(), policy);
        }
    }

    /// Check if this config has any overrides.
    pub fn is_empty(&self) -> bool {
        self.policies.is_empty()
    }
}

impl Default for PermissionPolicyConfig {
    fn default() -> Self {
        Self::new()
    }
}

/// Permission monitoring metrics (S5.7).
///
/// Tracks cache hit rates, request latency, and per-category statistics
/// for the Gateway's permission checking system.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PermissionMetrics {
    /// Total number of permission check requests processed
    pub total_checks: u64,
    /// Number of checks resolved from cache (without DB query)
    pub cache_hits: u64,
    /// Number of checks that required DB lookup
    pub cache_misses: u64,
    /// Cumulative check latency in microseconds
    pub total_latency_us: u64,
    /// Per-category check counts (e.g. "network" → 42)
    #[serde(default)]
    pub by_category: HashMap<String, u64>,
}

impl PermissionMetrics {
    /// Create an empty metrics instance.
    pub fn new() -> Self {
        Self::default()
    }

    /// Record a permission check result.
    pub fn record_check(&mut self, perm: &Permission, cache_hit: bool, latency_us: u64) {
        self.total_checks += 1;
        if cache_hit {
            self.cache_hits += 1;
        } else {
            self.cache_misses += 1;
        }
        self.total_latency_us += latency_us;
        *self.by_category.entry(perm.category().to_string()).or_insert(0) += 1;
    }

    /// Cache hit rate (0.0 to 1.0). Returns 0.0 if no checks recorded.
    pub fn cache_hit_rate(&self) -> f64 {
        if self.total_checks == 0 {
            return 0.0;
        }
        self.cache_hits as f64 / self.total_checks as f64
    }

    /// Average check latency in microseconds.
    pub fn avg_latency_us(&self) -> f64 {
        if self.total_checks == 0 {
            return 0.0;
        }
        self.total_latency_us as f64 / self.total_checks as f64
    }

    /// Reset all metrics to zero.
    pub fn reset(&mut self) {
        *self = Self::default();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_permission_parse_network() {
        let p = Permission::parse("network:https://api.weather.com").unwrap();
        assert!(matches!(p, Permission::Network(Some(_))));
        assert_eq!(p.to_permission_string(), "network:https://api.weather.com");
    }

    #[test]
    fn test_permission_parse_filesystem() {
        let p = Permission::parse("filesystem:read:~/Documents").unwrap();
        assert!(matches!(p, Permission::FilesystemRead(Some(_))));

        let p2 = Permission::parse("filesystem:write:~/workdir").unwrap();
        assert!(matches!(p2, Permission::FilesystemWrite(Some(_))));
    }

    #[test]
    fn test_permission_parse_memory() {
        let p = Permission::parse("memory:read").unwrap();
        assert_eq!(p, Permission::MemoryRead);

        let p2 = Permission::parse("memory:write").unwrap();
        assert_eq!(p2, Permission::MemoryWrite);
    }

    #[test]
    fn test_permission_parse_shell() {
        let p = Permission::parse("shell").unwrap();
        assert_eq!(p, Permission::Shell);
    }

    #[test]
    fn test_permission_parse_intent() {
        let p = Permission::parse("intent:send:com.example.calendar").unwrap();
        assert!(matches!(p, Permission::IntentSend(Some(_))));

        let p2 = Permission::parse("intent:receive:com.example.weather").unwrap();
        assert!(matches!(p2, Permission::IntentReceive(Some(_))));
    }

    #[test]
    fn test_permission_parse_invalid() {
        assert!(Permission::parse("invalid").is_err());
        assert!(Permission::parse("filesystem:execute").is_err());
    }

    #[test]
    fn test_permission_matches_broad_narrow() {
        // Broad permission (no scope) matches narrow (with scope)
        let broad = Permission::Network(None);
        let narrow = Permission::Network(Some("https://api.weather.com".into()));
        assert!(broad.matches(&narrow));

        // Narrow doesn't match broad
        assert!(!narrow.matches(&broad));

        // Exact match
        let same = Permission::Network(Some("https://api.weather.com".into()));
        assert!(narrow.matches(&same));
    }

    #[test]
    fn test_permission_matches_different_types() {
        let network = Permission::Network(None);
        let shell = Permission::Shell;
        assert!(!network.matches(&shell));
    }

    #[test]
    fn test_permission_toml_roundtrip() {
        let perms = vec![
            Permission::Network(Some("https://api.weather.com".into())),
            Permission::MemoryRead,
            Permission::Shell,
        ];
        // Use JSON for Vec roundtrip (TOML array-of-tables format is verbose)
        let json_str = serde_json::to_string(&perms).unwrap();
        let parsed: Vec<Permission> = serde_json::from_str(&json_str).unwrap();
        assert_eq!(perms, parsed);
    }

    #[test]
    fn test_permission_category() {
        assert_eq!(Permission::Network(None).category(), "network");
        assert_eq!(Permission::FilesystemRead(None).category(), "filesystem");
        assert_eq!(Permission::MemoryRead.category(), "memory");
        assert_eq!(Permission::IntentSend(None).category(), "intent");
        assert_eq!(Permission::IdentityRead.category(), "identity");
        assert_eq!(Permission::Shell.category(), "shell");
        assert_eq!(Permission::Wasm.category(), "wasm");
    }

    // ── PermissionGrant tests ──────────────────────────────────────────

    #[test]
    fn test_permission_grant_new() {
        let grant = PermissionGrant::new(
            "com.example.weather",
            Permission::Network(Some("https://api.weather.com".into())),
            "user",
        );
        assert_eq!(grant.agent_id, "com.example.weather");
        assert_eq!(grant.authorized_by, "user");
        assert!(grant.expires_at.is_none());
        assert!(grant.scope.is_none());
        assert!(!grant.is_expired());
    }

    #[test]
    fn test_permission_grant_with_expiry() {
        let past = chrono::Utc::now().timestamp_millis() - 10000;
        let expired_grant = PermissionGrant::with_expiry(
            "com.example.weather",
            Permission::Network(None),
            "user",
            past,
        );
        assert!(expired_grant.is_expired());

        let future = chrono::Utc::now().timestamp_millis() + 86400000; // 24h from now
        let valid_grant = PermissionGrant::with_expiry(
            "com.example.weather",
            Permission::Network(None),
            "user",
            future,
        );
        assert!(!valid_grant.is_expired());
    }

    #[test]
    fn test_permission_grant_matches_request() {
        let grant = PermissionGrant::new(
            "com.example.weather",
            Permission::Network(None), // broad: all network
            "user",
        );
        // Broad grant matches narrow request
        assert!(grant.matches_request(&Permission::Network(Some("https://api.weather.com".into()))));
        // Same type doesn't match different type
        assert!(!grant.matches_request(&Permission::Shell));
    }

    #[test]
    fn test_permission_grant_expired_does_not_match() {
        let past = chrono::Utc::now().timestamp_millis() - 10000;
        let expired = PermissionGrant::with_expiry(
            "com.example.weather",
            Permission::Network(None),
            "user",
            past,
        );
        // Expired grant does not match any request
        assert!(!expired.matches_request(&Permission::Network(None)));
    }

    #[test]
    fn test_permission_grant_serialization() {
        let grant = PermissionGrant::new(
            "com.example.weather",
            Permission::Shell,
            "user",
        );
        let json = serde_json::to_string(&grant).unwrap();
        let parsed: PermissionGrant = serde_json::from_str(&json).unwrap();
        assert_eq!(grant, parsed);
    }

    // ── PermissionPolicy tests ─────────────────────────────────────────

    #[test]
    fn test_permission_policy_for_low_risk() {
        assert_eq!(PermissionPolicy::for_permission(&Permission::MemoryRead), PermissionPolicy::Allow);
        assert_eq!(PermissionPolicy::for_permission(&Permission::IdentityRead), PermissionPolicy::Allow);
        assert_eq!(PermissionPolicy::for_permission(&Permission::IntentReceive(None)), PermissionPolicy::Allow);
    }

    #[test]
    fn test_permission_policy_for_high_risk() {
        assert_eq!(PermissionPolicy::for_permission(&Permission::Shell), PermissionPolicy::AskAlways);
        assert_eq!(PermissionPolicy::for_permission(&Permission::IdentityWrite), PermissionPolicy::AskAlways);
        assert_eq!(PermissionPolicy::for_permission(&Permission::Wasm), PermissionPolicy::AskAlways);
    }

    #[test]
    fn test_permission_policy_for_medium_risk() {
        assert_eq!(PermissionPolicy::for_permission(&Permission::Network(None)), PermissionPolicy::Default);
        assert_eq!(PermissionPolicy::for_permission(&Permission::FilesystemWrite(None)), PermissionPolicy::Default);
    }

    // ── S5.6: PermissionPolicyConfig tests ─────────────────────────────

    #[test]
    fn test_policy_config_override_category() {
        let mut config = PermissionPolicyConfig::new();
        assert!(config.is_empty());

        // Override shell category to Deny
        config.set_policy("shell", PermissionPolicy::Deny);
        assert!(!config.is_empty());
        assert_eq!(
            config.get_policy(&Permission::Shell),
            Some(PermissionPolicy::Deny)
        );
    }

    #[test]
    fn test_for_permission_with_config_respects_override() {
        let mut config = PermissionPolicyConfig::new();

        // Without override: Shell defaults to AskAlways
        assert_eq!(
            PermissionPolicy::for_permission_with_config(&Permission::Shell, Some(&config)),
            PermissionPolicy::AskAlways
        );

        // With override: Shell becomes Deny
        config.set_policy("shell", PermissionPolicy::Deny);
        assert_eq!(
            PermissionPolicy::for_permission_with_config(&Permission::Shell, Some(&config)),
            PermissionPolicy::Deny
        );

        // No config: falls back to built-in default
        assert_eq!(
            PermissionPolicy::for_permission_with_config(&Permission::Shell, None),
            PermissionPolicy::AskAlways
        );
    }

    #[test]
    fn test_policy_config_set_default_removes_override() {
        let mut config = PermissionPolicyConfig::new();

        // Set an override
        config.set_policy("network", PermissionPolicy::Deny);
        assert!(!config.is_empty());

        // Setting to Default removes the override (hot-reload behavior)
        config.set_policy("network", PermissionPolicy::Default);
        assert!(config.is_empty());
        assert_eq!(config.get_policy(&Permission::Network(None)), None);

        // Now falls back to built-in
        assert_eq!(
            PermissionPolicy::for_permission_with_config(
                &Permission::Network(None),
                Some(&config)
            ),
            PermissionPolicy::Default
        );
    }

    // ── New Permission type tests ─────────────────────────────────────

    #[test]
    fn test_permission_parse_identity() {
        let p = Permission::parse("identity:read").unwrap();
        assert_eq!(p, Permission::IdentityRead);

        let p2 = Permission::parse("identity:write").unwrap();
        assert_eq!(p2, Permission::IdentityWrite);
    }

    #[test]
    fn test_permission_parse_wasm() {
        let p = Permission::parse("wasm").unwrap();
        assert_eq!(p, Permission::Wasm);
    }

    #[test]
    fn test_identity_wasm_toml_roundtrip() {
        let perms = vec![
            Permission::IdentityRead,
            Permission::IdentityWrite,
            Permission::Wasm,
        ];
        let json_str = serde_json::to_string(&perms).unwrap();
        let parsed: Vec<Permission> = serde_json::from_str(&json_str).unwrap();
        assert_eq!(perms, parsed);
    }

    // ── RagQuery permission tests (S4.6) ─────────────────────────────────

    #[test]
    fn test_permission_parse_rag_query() {
        let p = Permission::parse("rag:query").unwrap();
        assert_eq!(p, Permission::RagQuery(None));

        let p2 = Permission::parse("rag:query:https://rag.corp.example.com").unwrap();
        assert_eq!(p2, Permission::RagQuery(Some("https://rag.corp.example.com".into())));
    }

    #[test]
    fn test_rag_query_permission_string() {
        assert_eq!(Permission::RagQuery(None).to_permission_string(), "rag:query");
        assert_eq!(
            Permission::RagQuery(Some("https://rag.corp.example.com".into())).to_permission_string(),
            "rag:query:https://rag.corp.example.com"
        );
    }

    #[test]
    fn test_rag_query_matches_broad_narrow() {
        let broad = Permission::RagQuery(None);
        let narrow = Permission::RagQuery(Some("https://rag.corp.example.com".into()));
        assert!(broad.matches(&narrow));
        assert!(!narrow.matches(&broad));
    }

    #[test]
    fn test_rag_query_toml_roundtrip() {
        let perms = vec![
            Permission::RagQuery(None),
            Permission::RagQuery(Some("https://rag.corp.example.com".into())),
        ];
        let json_str = serde_json::to_string(&perms).unwrap();
        let parsed: Vec<Permission> = serde_json::from_str(&json_str).unwrap();
        assert_eq!(perms, parsed);
    }

    #[test]
    fn test_rag_query_category() {
        assert_eq!(Permission::RagQuery(None).category(), "rag");
    }

    #[test]
    fn test_rag_query_policy_default() {
        assert_eq!(PermissionPolicy::for_permission(&Permission::RagQuery(None)), PermissionPolicy::Default);
    }

    // ── S5.2: PermissionParseError tests ──────────────────────────────────

    #[test]
    fn test_parse_error_unknown_category() {
        let err = Permission::parse("foobar:baz").unwrap_err();
        assert!(err.reason.contains("unknown category"));
        assert!(err.reason.contains("foobar"));
        assert_eq!(err.input, "foobar:baz");
    }

    #[test]
    fn test_parse_error_missing_colon() {
        let err = Permission::parse("network").unwrap_err();
        assert!(err.reason.contains("missing category delimiter"));
    }

    #[test]
    fn test_parse_error_invalid_filesystem_access() {
        let err = Permission::parse("filesystem:execute:/tmp").unwrap_err();
        assert!(err.reason.contains("invalid filesystem access"));
        assert!(err.reason.contains("execute"));
    }

    #[test]
    fn test_parse_error_missing_path() {
        let err = Permission::parse("filesystem:read").unwrap_err();
        assert!(err.reason.contains("missing"));
    }

    #[test]
    fn test_parse_error_display() {
        let err = Permission::parse("foobar:baz").unwrap_err();
        let display = format!("{}", err);
        assert!(display.contains("foobar:baz"));
        assert!(display.contains("unknown category"));
    }

    // ── S5.7: PermissionMetrics tests ──────────────────────────────────

    #[test]
    fn test_metrics_record_and_calculate() {
        let mut metrics = PermissionMetrics::new();
        assert_eq!(metrics.total_checks, 0);
        assert_eq!(metrics.cache_hit_rate(), 0.0);
        assert_eq!(metrics.avg_latency_us(), 0.0);

        // Record two cache hits, one cache miss
        metrics.record_check(&Permission::Network(None), true, 100);
        metrics.record_check(&Permission::MemoryRead, false, 250);
        metrics.record_check(&Permission::Network(None), true, 50);

        assert_eq!(metrics.total_checks, 3);
        assert_eq!(metrics.cache_hits, 2);
        assert_eq!(metrics.cache_misses, 1);
        assert!((metrics.cache_hit_rate() - 2.0 / 3.0).abs() < 0.001);
        assert!((metrics.avg_latency_us() - 400.0 / 3.0).abs() < 0.001);
    }

    #[test]
    fn test_metrics_by_category() {
        let mut metrics = PermissionMetrics::new();

        metrics.record_check(&Permission::Network(None), true, 10);
        metrics.record_check(&Permission::Network(None), false, 20);
        metrics.record_check(&Permission::Shell, true, 5);
        metrics.record_check(&Permission::MemoryRead, false, 15);

        assert_eq!(metrics.by_category.get("network"), Some(&2));
        assert_eq!(metrics.by_category.get("shell"), Some(&1));
        assert_eq!(metrics.by_category.get("memory"), Some(&1));
        assert_eq!(metrics.by_category.len(), 3);
    }

    #[test]
    fn test_metrics_reset() {
        let mut metrics = PermissionMetrics::new();
        metrics.record_check(&Permission::Shell, true, 100);
        metrics.record_check(&Permission::Network(None), false, 200);

        assert_eq!(metrics.total_checks, 2);

        metrics.reset();
        assert_eq!(metrics.total_checks, 0);
        assert_eq!(metrics.cache_hits, 0);
        assert_eq!(metrics.cache_misses, 0);
        assert_eq!(metrics.total_latency_us, 0);
        assert!(metrics.by_category.is_empty());
    }

    // ── S5.5: Bincode serialization tests ───────────────────────────────

    #[test]
    fn test_permission_grant_bincode_roundtrip() {
        let grant = PermissionGrant::new(
            "com.example.weather",
            Permission::Network(Some("https://api.weather.com".into())),
            "user",
        );
        let bin = grant.to_bincode().unwrap();
        let decoded = PermissionGrant::from_bincode(&bin).unwrap();
        assert_eq!(grant, decoded);
    }

    #[test]
    fn test_permission_grant_bincode_smaller_than_json() {
        let grant = PermissionGrant::new(
            "com.example.weather",
            Permission::Network(Some("https://api.weather.com".into())),
            "user",
        );
        let bin = grant.to_bincode().unwrap();
        let json = grant.to_json().unwrap();
        // Bincode should be more compact than JSON for structured data
        assert!(bin.len() <= json.len(),
            "bincode size {} > json size {}", bin.len(), json.len());
    }

    #[test]
    fn test_permission_grant_json_backward_compat() {
        let grant = PermissionGrant::with_expiry(
            "com.example.agent",
            Permission::Shell,
            "admin",
            1700000000000,
        );
        // JSON roundtrip still works (backward compatibility)
        let json = grant.to_json().unwrap();
        let decoded = PermissionGrant::from_json(&json).unwrap();
        assert_eq!(grant, decoded);
        
        // Bincode also works for the same grant
        let bin = grant.to_bincode().unwrap();
        let decoded_bin = PermissionGrant::from_bincode(&bin).unwrap();
        assert_eq!(grant, decoded_bin);
    }
}
