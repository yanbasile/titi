//! Simple Redis-like protocol parser
//!
//! Implements a simplified protocol for command parsing and response serialization.

use serde_json;

#[derive(Debug, Clone)]
pub enum Response {
    Ok,
    OkWithData(String),
    Error(String),
    String(String),
    Array(Vec<String>),
    Json(serde_json::Value),
}

impl Response {
    /// Serialize response to wire format
    pub fn serialize(&self) -> String {
        match self {
            Response::Ok => "+OK\n".to_string(),
            Response::OkWithData(data) => format!("+OK {}\n", data),
            Response::Error(msg) => format!("-ERR {}\n", msg),
            Response::String(s) => format!("\"{}\"\n", s.replace('"', "\\\"")),
            Response::Array(arr) => {
                let items: Vec<String> = arr.iter().map(|s| format!("\"{}\"", s)).collect();
                format!("[{}]\n", items.join(", "))
            }
            Response::Json(value) => format!("{}\n", serde_json::to_string(value).unwrap()),
        }
    }
}

pub struct Protocol;

impl Protocol {
    /// Parse a command line into command and arguments
    pub fn parse_command(line: &str) -> Result<(String, Vec<String>), String> {
        let line = line.trim();
        if line.is_empty() {
            return Err("Empty command".to_string());
        }

        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.is_empty() {
            return Err("Empty command".to_string());
        }

        let command = parts[0].to_uppercase();
        let args = parts[1..].iter().map(|s| s.to_string()).collect();

        Ok((command, args))
    }

    /// Parse a quoted string argument
    pub fn parse_quoted_string(s: &str) -> String {
        if s.starts_with('"') && s.ends_with('"') {
            s[1..s.len() - 1].replace("\\\"", "\"")
        } else {
            s.to_string()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_command() {
        let (cmd, args) = Protocol::parse_command("AUTH token123").unwrap();
        assert_eq!(cmd, "AUTH");
        assert_eq!(args, vec!["token123"]);

        let (cmd, args) = Protocol::parse_command("LIST SESSIONS").unwrap();
        assert_eq!(cmd, "LIST");
        assert_eq!(args, vec!["SESSIONS"]);

        let (cmd, args) = Protocol::parse_command("INJECT session-1/pane-1 \"echo hello\"").unwrap();
        assert_eq!(cmd, "INJECT");
        // Note: Basic parser splits on whitespace, quotes not preserved
        assert_eq!(args.len(), 3);
        assert_eq!(args[0], "session-1/pane-1");
        assert_eq!(args[1], "\"echo");
        assert_eq!(args[2], "hello\"");
    }

    #[test]
    fn test_response_serialization() {
        assert_eq!(Response::Ok.serialize(), "+OK\n");
        assert_eq!(
            Response::OkWithData("test".to_string()).serialize(),
            "+OK test\n"
        );
        assert_eq!(
            Response::Error("failed".to_string()).serialize(),
            "-ERR failed\n"
        );
        assert_eq!(
            Response::String("hello".to_string()).serialize(),
            "\"hello\"\n"
        );
    }

    #[test]
    fn test_array_serialization() {
        let arr = vec!["session1".to_string(), "session2".to_string()];
        let response = Response::Array(arr);
        assert_eq!(response.serialize(), "[\"session1\", \"session2\"]\n");
    }
}
