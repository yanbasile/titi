# TITI Security Analysis

**Project**: TITI Terminal Emulator + Redititi Automation Server
**Version**: 0.1.0
**Analysis Date**: 2025-12-15
**Analyst**: SECURITY_AGENT
**Risk Level**: HIGH

---

## Table of Contents

1. [Executive Summary](#executive-summary)
2. [Architecture Overview](#architecture-overview)
3. [Threat Model](#threat-model)
4. [Security Findings](#security-findings)
5. [Vulnerability Analysis](#vulnerability-analysis)
6. [Attack Vectors](#attack-vectors)
7. [Recommendations](#recommendations)
8. [Security Checklist](#security-checklist)

---

## Executive Summary

TITI is a GPU-accelerated terminal emulator with a Redis-like automation server (Redititi) designed for orchestrating multiple Claude Code agents. The project has **significant security implications** due to:

1. **Command Injection Capability**: Redititi is explicitly designed to inject commands into terminal sessions
2. **Network-Accessible Server**: TCP server that accepts connections and executes commands
3. **PTY Access**: Direct access to pseudo-terminal with shell execution
4. **Multi-Agent Orchestration**: Designed to control multiple AI agents simultaneously

### Overall Risk Assessment

| Component | Risk Level | Primary Concern |
|-----------|------------|-----------------|
| **Redititi Server** | HIGH | Command injection, token leakage, no encryption |
| **Authentication** | MEDIUM | Single global token, timing attacks, no rotation |
| **PTY Handling** | HIGH | Arbitrary shell execution, no sandboxing |
| **Protocol Parser** | LOW | Simple protocol, limited attack surface |
| **Terminal Parser** | LOW | Uses battle-tested VTE library |

### Key Findings Summary

| ID | Severity | Finding |
|----|----------|---------|
| SEC-001 | CRITICAL | Token logged to console at startup |
| SEC-002 | HIGH | No TLS encryption on TCP connection |
| SEC-003 | HIGH | No rate limiting on commands/connections |
| SEC-004 | MEDIUM | Timing attack possible on token comparison |
| SEC-005 | MEDIUM | No queue depth limits (DoS vector) |
| SEC-006 | MEDIUM | Shell spawned from $SHELL env var |
| SEC-007 | LOW | No audit logging of commands |

---

## Architecture Overview

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                        TITI + REDITITI ARCHITECTURE                          │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                              │
│   External Client                                                            │
│   (Python/netcat/etc)                                                        │
│         │                                                                    │
│         │ TCP (plaintext)                                                    │
│         │ Port 6379                                                          │
│         ▼                                                                    │
│   ┌──────────────────────────────────────────────────────────────────────┐  │
│   │                     REDITITI SERVER                                   │  │
│   │                                                                       │  │
│   │  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐ │  │
│   │  │ TCP Server  │  │ Auth System │  │  Registry   │  │  Channels   │ │  │
│   │  │ (tokio)     │  │ (token)     │  │ (sessions)  │  │  (pub/sub)  │ │  │
│   │  └──────┬──────┘  └──────┬──────┘  └──────┬──────┘  └──────┬──────┘ │  │
│   │         │                │                │                │        │  │
│   │         └────────────────┴────────────────┴────────────────┘        │  │
│   │                                   │                                  │  │
│   │                          Command Handler                             │  │
│   │                                   │                                  │  │
│   │         ┌─────────────────────────┴─────────────────────────┐       │  │
│   │         │                                                    │       │  │
│   │         ▼                                                    ▼       │  │
│   │    INJECT Command                                    CAPTURE Command │  │
│   │    (writes to PTY)                                   (reads output)  │  │
│   │                                                                       │  │
│   └───────────────────────────────────┬──────────────────────────────────┘  │
│                                       │                                      │
│                                       │ Input/Output Channels                │
│                                       ▼                                      │
│   ┌──────────────────────────────────────────────────────────────────────┐  │
│   │                        TITI TERMINAL                                  │  │
│   │                                                                       │  │
│   │  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐ │  │
│   │  │    PTY      │  │    Grid     │  │   Parser    │  │  Renderer   │ │  │
│   │  │ (shell)     │  │ (buffer)    │  │   (VTE)     │  │   (wgpu)    │ │  │
│   │  └──────┬──────┘  └─────────────┘  └─────────────┘  └─────────────┘ │  │
│   │         │                                                            │  │
│   │         ▼                                                            │  │
│   │    /bin/bash or $SHELL                                               │  │
│   │    (full shell access)                                               │  │
│   │                                                                       │  │
│   └──────────────────────────────────────────────────────────────────────┘  │
│                                                                              │
└─────────────────────────────────────────────────────────────────────────────┘
```

### Security Boundaries

| Boundary | Description | Protection |
|----------|-------------|------------|
| Network → Server | TCP connection from client | Token authentication, localhost binding |
| Server → Terminal | Command injection via channels | Channel naming convention |
| Terminal → Shell | PTY execution | None (by design) |

---

## Threat Model

### Threat Actors

1. **Malicious Local User**
   - Capability: Access to same machine, can read token file
   - Motivation: Unauthorized command execution, data theft
   - Risk: HIGH (token file accessible, no user isolation)

2. **Network Attacker (Same Network)**
   - Capability: Network sniffing, MITM
   - Motivation: Credential theft, command injection
   - Risk: MEDIUM (localhost binding mitigates, but no encryption)

3. **Malicious Client Library**
   - Capability: Uses legitimate authentication
   - Motivation: Execute malicious commands, pivot attack
   - Risk: HIGH (no command filtering once authenticated)

4. **Compromised Orchestration Script**
   - Capability: Full control over terminal agents
   - Motivation: Data exfiltration, system compromise
   - Risk: CRITICAL (designed for full shell access)

### Assets to Protect

| Asset | Sensitivity | Current Protection |
|-------|-------------|-------------------|
| Authentication Token | CRITICAL | File permissions (0o600) |
| Shell Access | CRITICAL | Token authentication only |
| Terminal Output | HIGH | Channel-based access |
| Session State | MEDIUM | Server memory |
| User Data (in terminals) | HIGH | None (passes through) |

---

## Security Findings

### SEC-001: Token Logged to Console (CRITICAL)

**Location**: `src/bin/redititi.rs:69`

```rust
log::info!("Token:      {}", auth.token());
```

**Impact**: Authentication token is logged to stdout at server startup. Anyone who can view logs (systemd journal, redirected output, log aggregators) gains full access to the server.

**CVSS Score**: 8.1 (High)

**Recommendation**: Remove token logging or mask it (e.g., show only first/last 4 characters).

---

### SEC-002: No TLS Encryption (HIGH)

**Location**: `src/server/tcp_server.rs`

**Description**: TCP server operates in plaintext. All commands, including the AUTH token, are transmitted unencrypted.

```rust
let listener = TcpListener::bind(&self.addr).await?;
// No TLS wrapper
```

**Impact**:
- Token can be sniffed on network
- Commands can be intercepted/modified (MITM)
- Session hijacking possible

**Recommendation**: Implement TLS using `tokio-rustls` or `tokio-native-tls`.

---

### SEC-003: No Rate Limiting (HIGH)

**Location**: `src/server/tcp_server.rs`, `src/server/commands.rs`

**Description**: No limits on:
- Connection attempts
- Authentication attempts per IP (only per-connection)
- Commands per second
- Sessions/panes creation

**Impact**:
- Brute force attacks on token
- Denial of service via resource exhaustion
- Memory exhaustion via queue flooding

**Recommendation**: Implement rate limiting at multiple levels:
- Connection rate per IP
- Auth attempts per IP (persistent)
- Commands per connection per second
- Maximum sessions/panes

---

### SEC-004: Timing Attack on Token Comparison (MEDIUM)

**Location**: `src/server/auth.rs:77-79`

```rust
pub fn validate(&self, token: &str) -> bool {
    self.token == token  // Not constant-time!
}
```

**Impact**: String comparison exits early on mismatch, allowing timing-based attacks to guess the token byte-by-byte.

**Recommendation**: Use constant-time comparison:

```rust
use subtle::ConstantTimeEq;

pub fn validate(&self, token: &str) -> bool {
    self.token.as_bytes().ct_eq(token.as_bytes()).into()
}
```

---

### SEC-005: No Queue Depth Limits (MEDIUM)

**Location**: `src/server/channels.rs`

```rust
pub async fn publish(&self, channel_name: &str, content: String) -> usize {
    // ...
    channel.queue.push_back(message);  // No limit!
    // ...
}
```

**Impact**: Attacker can flood channels with messages, exhausting server memory.

**Recommendation**: Implement queue limits:
- Maximum messages per channel
- Maximum total messages
- Message size limits

---

### SEC-006: Shell from Environment Variable (MEDIUM)

**Location**: `src/terminal/pty.rs:76`

```rust
if let Ok(shell) = std::env::var("SHELL") {
    return (shell, vec![]);
}
```

**Impact**: If attacker can set $SHELL environment variable, they control what shell is spawned (could point to malicious binary).

**Recommendation**:
- Validate $SHELL against known safe shells
- Use absolute paths from `/etc/shells`
- Consider hardcoding safe defaults

---

### SEC-007: No Audit Logging (LOW)

**Location**: Throughout codebase

**Description**: Commands are not logged for audit purposes. No record of:
- Who executed what commands
- When commands were executed
- Which terminals were accessed

**Recommendation**: Implement audit logging:
- Log all commands with timestamps
- Log session creation/destruction
- Log authentication successes/failures with source IP

---

## Vulnerability Analysis

### Input Validation

| Input Point | Validation | Risk |
|-------------|------------|------|
| AUTH token | Exact match | LOW |
| Command parsing | split_whitespace | MEDIUM (no quoted string support) |
| Channel names | None | LOW (used as keys only) |
| INJECT command | None | HIGH (by design) |
| PUBLISH content | None | LOW |

### Memory Safety

Rust provides memory safety by default. Key observations:

- No `unsafe` blocks in core server code
- Uses `Arc<RwLock<>>` for safe concurrent access
- VTE parser is well-tested
- Buffer sizes bounded in PTY reads

### Concurrency Safety

- Tokio async runtime handles concurrent connections
- RwLock prevents data races on shared state
- Connection IDs are atomic counters

---

## Attack Vectors

### AV-001: Token Brute Force

**Description**: Attacker attempts to guess the 64-character alphanumeric token.

**Feasibility**: LOW (62^64 combinations, 3 attempts per connection)

**Current Mitigations**:
- 64-character token (good entropy)
- 3 attempts per connection

**Additional Mitigations Needed**:
- Rate limiting per IP
- Increasing delays between attempts
- Account lockout after N failures

---

### AV-002: Network Sniffing

**Description**: Attacker on same network captures token during AUTH.

**Feasibility**: HIGH (plaintext TCP)

**Current Mitigations**:
- Localhost binding (127.0.0.1)

**Additional Mitigations Needed**:
- TLS encryption
- Token refresh mechanism

---

### AV-003: Malicious Command Injection

**Description**: Authenticated attacker injects malicious shell commands.

**Feasibility**: HIGH (by design - this is the intended use case)

**Current Mitigations**:
- Token authentication

**Additional Mitigations Needed**:
- Command allowlist (optional mode)
- Sandboxing (optional mode)
- Audit logging

---

### AV-004: Resource Exhaustion DoS

**Description**: Attacker creates many sessions/panes or floods channels.

**Feasibility**: HIGH

**Attack Steps**:
1. Authenticate with valid token
2. Create unlimited sessions: `CREATE SESSION` in loop
3. Flood channels: `PUBLISH channel-X message` repeatedly
4. Server memory exhausted

**Mitigations Needed**:
- Session/pane limits per token
- Queue depth limits
- Message size limits
- Connection limits

---

### AV-005: Timing Side Channel

**Description**: Attacker uses timing differences to guess token.

**Feasibility**: MEDIUM (requires precise timing measurements)

**Attack Steps**:
1. Measure response time for `AUTH aaaa...`
2. If first char wrong, measure `AUTH baaa...`
3. Find char with longest response time
4. Repeat for each position

**Mitigations Needed**:
- Constant-time comparison
- Random delays on failed auth

---

### AV-006: Environment Variable Manipulation

**Description**: Attacker sets malicious $SHELL before PTY spawn.

**Feasibility**: LOW (requires local access before server start)

**Attack Steps**:
1. Set `SHELL=/tmp/malicious_shell`
2. Start TITI terminal
3. All PTYs spawn attacker's shell

**Mitigations Needed**:
- Validate $SHELL
- Use shells from `/etc/shells` only

---

## Recommendations

### Immediate (Pre-Production)

1. **Remove Token Logging**
   ```rust
   // Instead of logging full token:
   log::info!("Token: {}...{}", &auth.token()[..4], &auth.token()[60..]);
   ```

2. **Add Constant-Time Comparison**
   - Add `subtle` crate dependency
   - Use `ct_eq` for token validation

3. **Implement Basic Rate Limiting**
   - Use tokio-based rate limiter
   - Limit connections per IP
   - Limit commands per connection

### Short-Term (Before Public Release)

4. **Add TLS Support**
   - Implement with `tokio-rustls`
   - Support self-signed certs for local use
   - Support custom CA for enterprise use

5. **Add Resource Limits**
   - Maximum sessions per token
   - Maximum panes per session
   - Maximum queue depth
   - Maximum message size

6. **Add Audit Logging**
   - Log all commands with timestamps
   - Structured logging (JSON)
   - Configurable log destination

### Long-Term (Production Hardening)

7. **Per-User Authentication**
   - Support multiple tokens with different permissions
   - Token expiration
   - Token revocation

8. **Command Filtering Mode**
   - Optional allowlist mode
   - Regex-based command validation
   - Deny-by-default option

9. **Sandboxing Options**
   - Container-based isolation
   - Firejail/Bubblewrap integration
   - Capability dropping

---

## Security Checklist

### Authentication & Authorization

- [x] Token-based authentication implemented
- [x] Token file has restricted permissions (0o600)
- [ ] Constant-time token comparison
- [ ] Token rotation mechanism
- [ ] Per-user authentication
- [ ] Token expiration

### Network Security

- [x] Server binds to localhost by default
- [ ] TLS encryption supported
- [ ] Certificate validation
- [ ] Rate limiting implemented

### Input Validation

- [x] Command parsing implemented
- [ ] Command allowlist mode
- [ ] Message size limits
- [ ] Channel name validation

### Resource Management

- [ ] Connection limits
- [ ] Session/pane limits
- [ ] Queue depth limits
- [ ] Memory limits

### Logging & Monitoring

- [x] Basic logging implemented
- [ ] Sensitive data masked in logs
- [ ] Audit logging for commands
- [ ] Security event alerting

### Error Handling

- [x] Graceful error responses
- [x] Connection cleanup on disconnect
- [ ] No stack traces in production
- [ ] Error rate monitoring

---

## Dependencies Security

| Dependency | Version | Known Vulnerabilities | Notes |
|------------|---------|----------------------|-------|
| tokio | 1.0 | None known | Async runtime |
| wgpu | 23.0 | None known | GPU rendering |
| vte | 0.13 | None known | Terminal parser |
| portable-pty | 0.8 | None known | PTY management |
| rand | 0.8 | None known | Token generation |
| serde_json | 1.0 | None known | JSON parsing |

**Recommendation**: Set up `cargo-audit` in CI/CD to monitor for new vulnerabilities.

---

## Conclusion

TITI/Redititi is a powerful terminal automation system with legitimate use cases for AI agent orchestration. However, the current implementation has several security gaps that should be addressed before production use:

**Critical Issues**:
1. Token logged in plaintext (SEC-001)
2. No TLS encryption (SEC-002)

**Important Issues**:
3. No rate limiting (SEC-003)
4. Timing attack vulnerability (SEC-004)
5. No resource limits (SEC-005)

The localhost binding provides significant protection against remote attacks, but local privilege escalation and insider threats remain concerns. For production deployment, especially in multi-tenant environments, the recommendations in this document should be implemented.

---

## Document Control

| Version | Date | Author | Changes |
|---------|------|--------|---------|
| 1.0 | 2025-12-15 | SECURITY_AGENT | Initial security analysis |
