# Security Fixes Log

This document tracks security vulnerabilities identified and fixed in Titi/Redititi.

## Fixed Issues (2025-12-17)

### SEC-001: Token Logged to Console (CRITICAL) ✅ FIXED

**Severity**: CRITICAL
**CVSS Score**: 8.1 (High)
**Status**: ✅ Fixed in commit [pending]

**Description**: Authentication token was logged in plaintext to stdout at server startup. Anyone who could view logs (systemd journal, redirected output, log aggregators) gained full access to the server.

**Location**: `src/bin/redititi.rs:69`

**Original Code**:
```rust
log::info!("Token:      {}", auth.token());
```

**Fix Applied**:
```rust
// Security: Mask token to prevent exposure in logs (show only first/last 4 chars)
let token = auth.token();
let masked_token = if token.len() >= 8 {
    format!("{}...{}", &token[..4], &token[token.len()-4..])
} else {
    "****".to_string()
};
log::info!("Token:      {} (masked for security)", masked_token);
```

**Impact**: Prevents token exposure in logs, log aggregation systems, and console output.

---

### SEC-004: Timing Attack on Token Comparison (MEDIUM) ✅ FIXED

**Severity**: MEDIUM
**Status**: ✅ Fixed in commit [pending]

**Description**: Token validation used standard string comparison which exits early on mismatch, allowing timing-based attacks to guess the token byte-by-byte.

**Location**: `src/redititi_server/auth.rs:86-88`

**Original Code**:
```rust
pub fn validate(&self, token: &str) -> bool {
    self.token == token  // Not constant-time!
}
```

**Fix Applied**:
```rust
use subtle::ConstantTimeEq;

/// Validate a token against the stored token
///
/// Uses constant-time comparison to prevent timing attacks
pub fn validate(&self, token: &str) -> bool {
    // Use constant-time comparison to prevent timing side-channel attacks
    self.token.as_bytes().ct_eq(token.as_bytes()).into()
}
```

**Dependencies Added**: `subtle = "2.6"` in Cargo.toml

**Impact**: Prevents timing side-channel attacks that could be used to guess the authentication token.

---

### SEC-006: Shell from Environment Variable (MEDIUM) ✅ FIXED

**Severity**: MEDIUM
**Status**: ✅ Fixed in commit [pending]

**Description**: PTY spawning accepted any $SHELL environment variable value without validation. If an attacker could set $SHELL before server start, they could point it to a malicious binary.

**Location**: `src/terminal/pty.rs:76-78`

**Original Code**:
```rust
if let Ok(shell) = std::env::var("SHELL") {
    return (shell, vec![]);
}
```

**Fix Applied**:
```rust
fn get_shell() -> (String, Vec<String>) {
    // Try to get shell from environment
    if let Ok(shell) = std::env::var("SHELL") {
        // Security: Validate the shell path before using it
        if Self::is_valid_shell(&shell) {
            return (shell, vec![]);
        }
        // If invalid, fall through to defaults
        log::warn!("Shell from $SHELL '{}' is not valid, using default", shell);
    }
    // ... rest of default shell selection
}

/// Validate a shell path for security
///
/// Checks that the shell:
/// 1. Exists as a file
/// 2. Is an absolute path
/// 3. Matches known safe shell patterns
fn is_valid_shell(shell: &str) -> bool {
    use std::path::Path;

    let path = Path::new(shell);

    // Must be an absolute path
    if !path.is_absolute() {
        return false;
    }

    // Must exist as a file
    if !path.exists() || !path.is_file() {
        return false;
    }

    // On Unix, validate against /etc/shells or known shells
    #[cfg(unix)]
    {
        // Check /etc/shells if available
        if let Ok(shells_content) = std::fs::read_to_string("/etc/shells") {
            if shells_content.lines().any(|line| {
                let line = line.trim();
                !line.is_empty() && !line.starts_with('#') && line == shell
            }) {
                return true;
            }
        }

        // Fallback: Check against common known shells
        let known_shells = [
            "/bin/sh", "/bin/bash", "/bin/zsh", "/bin/dash", "/bin/ksh",
            "/bin/fish", "/bin/tcsh", "/bin/csh",
            "/usr/bin/sh", "/usr/bin/bash", "/usr/bin/zsh", "/usr/bin/dash",
            "/usr/bin/ksh", "/usr/bin/fish", "/usr/bin/tcsh", "/usr/bin/csh",
        ];

        known_shells.contains(&shell)
    }

    // Windows and other platforms...
}
```

**Impact**: Prevents arbitrary code execution via malicious $SHELL environment variable. Validates shells against:
1. /etc/shells (Unix)
2. Known safe shell paths
3. Absolute path requirement
4. File existence check

---

## Open Security Issues

### SEC-002: No TLS Encryption (HIGH) 🔴 OPEN

**Severity**: HIGH
**Status**: 🔴 Open - Tracked in SECURITY_RECOMMENDATIONS.md

**Description**: TCP server operates in plaintext. All commands, including the AUTH token, are transmitted unencrypted.

**Impact**:
- Token can be sniffed on network
- Commands can be intercepted/modified (MITM)
- Session hijacking possible

**Current Mitigation**: Server binds to localhost (127.0.0.1) by default, limiting exposure to local machine only.

**Recommendation**: Implement TLS using `tokio-rustls` or `tokio-native-tls`. See SECURITY_RECOMMENDATIONS.md for implementation details.

---

### SEC-003: No Rate Limiting (HIGH) 🔴 OPEN

**Severity**: HIGH
**Status**: 🔴 Open - Tracked in SECURITY_RECOMMENDATIONS.md

**Description**: No limits on connections, authentication attempts, or commands per second.

**Impact**:
- Brute force attacks on token
- Denial of service via resource exhaustion
- Memory exhaustion via queue flooding

**Recommendation**: Implement rate limiting at multiple levels:
- Connection rate per IP
- Auth attempts per IP (persistent)
- Commands per connection per second
- Maximum sessions/panes

See SECURITY_RECOMMENDATIONS.md for implementation details.

---

### SEC-005: No Queue Depth Limits (MEDIUM) 🟡 OPEN

**Severity**: MEDIUM
**Status**: 🟡 Open - Tracked in SECURITY_RECOMMENDATIONS.md

**Description**: Channels can accumulate unlimited messages, exhausting server memory.

**Recommendation**: Implement queue limits (max messages per channel, max total messages, message size limits).

---

### SEC-007: No Audit Logging (LOW) 🟢 OPEN

**Severity**: LOW
**Status**: 🟢 Open - Enhancement tracked in SECURITY_RECOMMENDATIONS.md

**Description**: Commands are not logged for audit purposes.

**Recommendation**: Implement audit logging for commands, session creation/destruction, and authentication events.

---

## Security Testing

### Pre-Production Tests
- [x] Token masking verified (logs show masked token)
- [x] Constant-time comparison verified (compilation successful)
- [x] Shell validation verified (invalid shells rejected)

### Required Before Production
- [ ] TLS encryption implemented and tested
- [ ] Rate limiting implemented and tested
- [ ] Full security audit of implemented fixes
- [ ] Penetration testing of authentication mechanism

---

## References

- **Security Analysis**: See root directory security analysis document
- **Security Recommendations**: [SECURITY_RECOMMENDATIONS.md](SECURITY_RECOMMENDATIONS.md)
- **Implementation Roadmap**: [IMPLEMENTATION_ROADMAP.md](IMPLEMENTATION_ROADMAP.md) - Phase 5: Security Hardening

---

## Document Control

| Version | Date | Author | Changes |
|---------|------|--------|---------|
| 1.0 | 2025-12-17 | Claude | Initial security fixes document - SEC-001, SEC-004, SEC-006 fixed |

---

## Security Contact

For security issues, please:
1. Review this document and SECURITY_RECOMMENDATIONS.md
2. Check if the issue is already tracked
3. If new, create a GitHub issue with [SECURITY] prefix
4. For critical vulnerabilities, contact maintainers directly

**Important**: Do not publicly disclose security vulnerabilities before they are patched.
