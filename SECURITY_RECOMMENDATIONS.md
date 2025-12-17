# Security Recommendations for Titi & Redititi

**Last Updated**: 2025-12-17
**Status**: Recommendations for Production Security Hardening

---

## 🔒 Executive Summary

This document provides comprehensive security recommendations for the Titi terminal emulator and Redititi automation server, with special focus on headless mode deployments and multi-agent orchestration scenarios.

### Current Security Status

✅ **Implemented**:
- Token-based authentication
- Session/pane isolation
- TCP connection management
- Basic input validation

⚠️ **Needs Enhancement**:
- Rate limiting
- Encryption (TLS/SSL)
- Advanced authentication (OAuth, JWT, mTLS)
- Audit logging
- Resource quotas
- Command sandboxing
- Input sanitization

---

## 🎯 High-Priority Security Enhancements

### 1. **TLS/SSL Encryption for Network Communication**

**Risk**: Unencrypted TCP communication exposes tokens and command data to network sniffing.

**Impact**: HIGH
**Effort**: MEDIUM
**Priority**: 🔴 **CRITICAL**

**Recommendation**:
```rust
// Add to redititi server configuration
[server]
tls_enabled = true
tls_cert_path = "/etc/redititi/cert.pem"
tls_key_path = "/etc/redititi/key.pem"
tls_ca_path = "/etc/redititi/ca.pem"  # For client certificate validation

// Example implementation in src/redititi_server/mod.rs
use tokio_rustls::{TlsAcceptor, rustls};

pub async fn run_with_tls(&self, tls_config: TlsConfig) -> Result<()> {
    let tls_acceptor = create_tls_acceptor(tls_config)?;
    let listener = TcpListener::bind(&self.addr).await?;

    loop {
        let (stream, addr) = listener.accept().await?;
        let tls_stream = tls_acceptor.accept(stream).await?;
        // Handle connection with tls_stream
    }
}
```

**Benefits**:
- Prevents token theft over network
- Protects command data and output
- Required for production deployments
- Enables compliance (GDPR, SOC 2, etc.)

---

### 2. **Rate Limiting & DoS Protection**

**Risk**: Malicious clients can overwhelm server with rapid requests, causing denial of service.

**Impact**: HIGH
**Effort**: LOW
**Priority**: 🔴 **CRITICAL**

**Recommendation**:
```rust
// Add rate limiter to src/redititi_server/rate_limiter.rs
use std::collections::HashMap;
use std::time::{Instant, Duration};
use tokio::sync::RwLock;

pub struct RateLimiter {
    // Track requests per connection
    requests: Arc<RwLock<HashMap<ConnectionId, RequestBucket>>>,
    max_requests_per_second: u32,
    max_burst: u32,
}

struct RequestBucket {
    tokens: u32,
    last_refill: Instant,
}

impl RateLimiter {
    pub async fn check_rate_limit(&self, conn_id: ConnectionId) -> Result<(), RateLimitError> {
        let mut requests = self.requests.write().await;
        let bucket = requests.entry(conn_id).or_insert_with(|| RequestBucket {
            tokens: self.max_burst,
            last_refill: Instant::now(),
        });

        // Refill tokens based on time elapsed
        let elapsed = bucket.last_refill.elapsed();
        let refill = (elapsed.as_secs_f64() * self.max_requests_per_second as f64) as u32;
        bucket.tokens = (bucket.tokens + refill).min(self.max_burst);
        bucket.last_refill = Instant::now();

        // Check if request allowed
        if bucket.tokens > 0 {
            bucket.tokens -= 1;
            Ok(())
        } else {
            Err(RateLimitError::TooManyRequests)
        }
    }
}

// Configuration in config.toml
[rate_limiting]
enabled = true
max_requests_per_second = 100
max_burst = 200
ban_duration_seconds = 300  # Ban for 5 minutes after repeated violations
```

**Benefits**:
- Prevents DoS attacks
- Protects server resources
- Ensures fair resource allocation
- Improves stability under load

---

### 3. **Advanced Authentication Options**

**Risk**: Simple token authentication is vulnerable if tokens are compromised.

**Impact**: HIGH
**Effort**: HIGH
**Priority**: 🟡 **HIGH**

**Recommendations**:

#### A. **OAuth 2.0 / OpenID Connect Integration**
```rust
// Add to src/redititi_server/auth.rs
use oauth2::{ClientId, ClientSecret, AuthUrl, TokenUrl};

pub struct OAuthAuth {
    client_id: ClientId,
    client_secret: ClientSecret,
    auth_url: AuthUrl,
    token_url: TokenUrl,
}

impl OAuthAuth {
    pub async fn validate_token(&self, token: &str) -> Result<UserInfo, AuthError> {
        // Validate token with OAuth provider
        // Return user identity and permissions
    }
}

// Configuration
[auth]
type = "oauth"
provider = "github"  # or "google", "okta", etc.
client_id = "your_client_id"
client_secret_env = "REDITITI_OAUTH_SECRET"  # Never hardcode secrets!
```

#### B. **JWT (JSON Web Tokens)**
```rust
use jsonwebtoken::{decode, Algorithm, Validation, DecodingKey};

pub struct JWTAuth {
    secret: String,
    algorithm: Algorithm,
}

impl JWTAuth {
    pub fn validate_jwt(&self, token: &str) -> Result<Claims, AuthError> {
        let validation = Validation::new(self.algorithm);
        let token_data = decode::<Claims>(
            token,
            &DecodingKey::from_secret(self.secret.as_ref()),
            &validation,
        )?;
        Ok(token_data.claims)
    }
}

// JWT Claims structure
#[derive(Serialize, Deserialize)]
struct Claims {
    sub: String,        // Subject (user ID)
    exp: usize,         // Expiration time
    iat: usize,         // Issued at
    permissions: Vec<String>,  // User permissions
}
```

#### C. **Mutual TLS (mTLS)**
```rust
// Require client certificates for authentication
[auth]
type = "mtls"
require_client_cert = true
client_ca_path = "/etc/redititi/client-ca.pem"
allowed_cn_patterns = ["agent-*", "user-*"]  # Certificate CN patterns
```

**Benefits**:
- Industry-standard authentication
- Token expiration and rotation
- Fine-grained permissions
- Audit trail of user actions
- Integration with existing identity providers

---

### 4. **Resource Quotas & Limits**

**Risk**: Malicious or buggy clients can exhaust server resources (memory, CPU, sessions).

**Impact**: MEDIUM
**Effort**: LOW
**Priority**: 🟡 **HIGH**

**Recommendation**:
```rust
// Add to src/redititi_server/quotas.rs
pub struct ResourceQuotas {
    max_sessions_per_client: usize,
    max_panes_per_session: usize,
    max_command_length: usize,
    max_output_buffer_size: usize,
    max_concurrent_connections: usize,
    max_memory_per_session_mb: usize,
}

impl ResourceQuotas {
    pub fn check_session_quota(&self, client_id: &str) -> Result<(), QuotaError> {
        let session_count = get_session_count(client_id);
        if session_count >= self.max_sessions_per_client {
            return Err(QuotaError::SessionLimitExceeded);
        }
        Ok(())
    }

    pub fn check_command_length(&self, cmd: &str) -> Result<(), QuotaError> {
        if cmd.len() > self.max_command_length {
            return Err(QuotaError::CommandTooLong);
        }
        Ok(())
    }
}

// Configuration
[quotas]
max_sessions_per_client = 100
max_panes_per_session = 50
max_command_length = 1048576  # 1MB
max_output_buffer_size = 10485760  # 10MB
max_concurrent_connections = 1000
max_memory_per_session_mb = 512
```

**Benefits**:
- Prevents resource exhaustion
- Fair resource allocation
- Predictable performance
- Easier capacity planning

---

### 5. **Audit Logging & Security Monitoring**

**Risk**: No visibility into security events, making incident response difficult.

**Impact**: MEDIUM
**Effort**: MEDIUM
**Priority**: 🟡 **HIGH**

**Recommendation**:
```rust
// Add comprehensive audit logging to src/redititi_server/audit.rs
use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};

#[derive(Serialize, Deserialize)]
pub struct AuditEvent {
    timestamp: DateTime<Utc>,
    event_type: AuditEventType,
    user_id: String,
    connection_id: ConnectionId,
    source_ip: String,
    session_id: Option<String>,
    command: Option<String>,
    result: EventResult,
    metadata: HashMap<String, String>,
}

#[derive(Serialize, Deserialize)]
pub enum AuditEventType {
    AuthenticationAttempt,
    AuthenticationSuccess,
    AuthenticationFailure,
    SessionCreated,
    SessionDestroyed,
    CommandInjected,
    ScreenCaptured,
    RateLimitViolation,
    QuotaExceeded,
    ConnectionEstablished,
    ConnectionClosed,
    ConfigurationChanged,
}

pub struct AuditLogger {
    log_file: String,
    syslog_enabled: bool,
    remote_logging: Option<RemoteLogConfig>,
}

impl AuditLogger {
    pub async fn log_event(&self, event: AuditEvent) {
        // Write to local file
        self.write_to_file(&event).await;

        // Send to syslog
        if self.syslog_enabled {
            self.send_to_syslog(&event);
        }

        // Send to remote logging service (Elasticsearch, Splunk, etc.)
        if let Some(config) = &self.remote_logging {
            self.send_to_remote(&event, config).await;
        }
    }
}

// Configuration
[audit]
enabled = true
log_file = "/var/log/redititi/audit.jsonl"
syslog_enabled = true
remote_logging_url = "https://logs.example.com/ingest"
log_authentication = true
log_commands = true  # Be careful with sensitive data!
log_rate_limits = true
```

**Benefits**:
- Security incident detection
- Forensic analysis capability
- Compliance requirements (SOC 2, PCI-DSS)
- Troubleshooting and debugging

---

### 6. **Input Validation & Command Sanitization**

**Risk**: Command injection vulnerabilities if user input is not properly validated.

**Impact**: CRITICAL
**Effort**: MEDIUM
**Priority**: 🔴 **CRITICAL**

**Recommendation**:
```rust
// Add to src/redititi_server/validation.rs
pub struct InputValidator {
    max_command_length: usize,
    allowed_commands_pattern: Regex,
    blocked_patterns: Vec<Regex>,
}

impl InputValidator {
    pub fn validate_command(&self, cmd: &str) -> Result<(), ValidationError> {
        // Check length
        if cmd.len() > self.max_command_length {
            return Err(ValidationError::CommandTooLong);
        }

        // Check for dangerous patterns
        for pattern in &self.blocked_patterns {
            if pattern.is_match(cmd) {
                return Err(ValidationError::DangerousPattern(pattern.to_string()));
            }
        }

        // Validate against allowed patterns (if restrictive mode)
        if !self.allowed_commands_pattern.is_match(cmd) {
            return Err(ValidationError::CommandNotAllowed);
        }

        Ok(())
    }

    pub fn sanitize_output(&self, output: &str) -> String {
        // Remove ANSI escape sequences that could be used for attacks
        // Strip control characters except newlines and tabs
        // Limit output length
        output
            .chars()
            .filter(|c| c.is_ascii_graphic() || *c == '\n' || *c == '\t')
            .take(self.max_output_length)
            .collect()
    }
}

// Configuration
[validation]
max_command_length = 1048576
max_output_length = 10485760
# Block dangerous shell patterns
blocked_patterns = [
    "rm -rf /",
    "; *(curl|wget)",  # Command injection
    "$(.*)",           # Command substitution
    "`.*`",            # Backtick execution
]
# Whitelist mode (optional)
allowed_commands = ["echo", "ls", "cat", "grep"]  # Only allow these
```

**Benefits**:
- Prevents command injection attacks
- Protects against privilege escalation
- Reduces attack surface
- Safe output handling

---

### 7. **Session Isolation & Sandboxing**

**Risk**: Sessions from different clients could interfere with each other.

**Impact**: HIGH
**Effort**: HIGH
**Priority**: 🟡 **HIGH**

**Recommendations**:

#### A. **Namespace Isolation**
```rust
// Use Linux namespaces for isolation
use nix::sched::{unshare, CloneFlags};
use nix::unistd::{chroot, setuid, setgid};

pub struct IsolatedSession {
    uid: u32,
    gid: u32,
    root_dir: PathBuf,
    network_isolated: bool,
}

impl IsolatedSession {
    pub fn spawn_isolated_pty(&self) -> Result<PTY> {
        // Create new namespaces
        unshare(
            CloneFlags::CLONE_NEWPID |   // Process namespace
            CloneFlags::CLONE_NEWNET |   // Network namespace
            CloneFlags::CLONE_NEWNS |    // Mount namespace
            CloneFlags::CLONE_NEWIPC     // IPC namespace
        )?;

        // Change root directory
        chroot(&self.root_dir)?;

        // Drop privileges
        setgid(self.gid)?;
        setuid(self.uid)?;

        // Spawn PTY with restricted environment
        self.create_pty()
    }
}
```

#### B. **seccomp-bpf Syscall Filtering**
```rust
use seccomp::{Context, Action, Rule};

pub fn apply_seccomp_filter() -> Result<()> {
    let mut ctx = Context::new(Action::Allow)?;

    // Block dangerous syscalls
    ctx.add_rule(Rule::new(libc::SYS_ptrace, Action::Errno(libc::EPERM)))?;
    ctx.add_rule(Rule::new(libc::SYS_mount, Action::Errno(libc::EPERM)))?;
    ctx.add_rule(Rule::new(libc::SYS_umount2, Action::Errno(libc::EPERM)))?;
    ctx.add_rule(Rule::new(libc::SYS_reboot, Action::Errno(libc::EPERM)))?;

    ctx.load()?;
    Ok(())
}
```

#### C. **cgroups Resource Limits**
```rust
pub struct CgroupLimits {
    memory_limit_mb: u64,
    cpu_shares: u64,
    pids_limit: u64,
}

impl CgroupLimits {
    pub fn apply_to_session(&self, session_id: &str) -> Result<()> {
        let cgroup_path = format!("/sys/fs/cgroup/redititi/{}", session_id);

        // Set memory limit
        std::fs::write(
            format!("{}/memory.max", cgroup_path),
            format!("{}", self.memory_limit_mb * 1024 * 1024)
        )?;

        // Set CPU shares
        std::fs::write(
            format!("{}/cpu.weight", cgroup_path),
            format!("{}", self.cpu_shares)
        )?;

        // Set PID limit
        std::fs::write(
            format!("{}/pids.max", cgroup_path),
            format!("{}", self.pids_limit)
        )?;

        Ok(())
    }
}
```

**Benefits**:
- Strong isolation between sessions
- Prevents privilege escalation
- Limits blast radius of compromised sessions
- Resource containment

---

### 8. **Encrypted Storage for Sensitive Data**

**Risk**: Tokens and session data stored in plaintext on disk.

**Impact**: MEDIUM
**Effort**: MEDIUM
**Priority**: 🟠 **MEDIUM**

**Recommendation**:
```rust
// Add to src/redititi_server/encryption.rs
use aes_gcm::{Aes256Gcm, Key, Nonce};
use aes_gcm::aead::{Aead, NewAead};

pub struct EncryptedStorage {
    cipher: Aes256Gcm,
    key_derivation_salt: Vec<u8>,
}

impl EncryptedStorage {
    pub fn new(master_password: &str) -> Result<Self> {
        // Derive key from password using PBKDF2
        let salt = generate_salt();
        let key = pbkdf2_derive_key(master_password, &salt, 100_000)?;

        Ok(Self {
            cipher: Aes256Gcm::new(Key::from_slice(&key)),
            key_derivation_salt: salt,
        })
    }

    pub fn encrypt_token(&self, token: &str) -> Result<Vec<u8>> {
        let nonce = generate_nonce();
        let ciphertext = self.cipher.encrypt(&nonce, token.as_bytes())?;
        Ok(ciphertext)
    }

    pub fn decrypt_token(&self, ciphertext: &[u8]) -> Result<String> {
        let nonce = extract_nonce(ciphertext);
        let plaintext = self.cipher.decrypt(&nonce, ciphertext)?;
        Ok(String::from_utf8(plaintext)?)
    }
}

// Configuration
[storage]
encryption_enabled = true
encryption_key_env = "REDITITI_ENCRYPTION_KEY"  # Never hardcode!
key_rotation_days = 90
```

**Benefits**:
- Protects tokens at rest
- Compliance with data protection regulations
- Defense in depth
- Key rotation capability

---

### 9. **Network Security & Firewall Rules**

**Risk**: Unauthorized network access to redititi server.

**Impact**: HIGH
**Effort**: LOW
**Priority**: 🟡 **HIGH**

**Recommendations**:

#### A. **Firewall Configuration** (iptables/nftables)
```bash
# Only allow connections from specific IPs
iptables -A INPUT -p tcp --dport 6379 -s 10.0.0.0/8 -j ACCEPT
iptables -A INPUT -p tcp --dport 6379 -j DROP

# Rate limit new connections
iptables -A INPUT -p tcp --dport 6379 -m state --state NEW -m recent --set
iptables -A INPUT -p tcp --dport 6379 -m state --state NEW -m recent --update --seconds 60 --hitcount 10 -j DROP
```

#### B. **IP Allowlisting**
```rust
// Add to src/redititi_server/access_control.rs
pub struct IPAccessControl {
    allowed_ips: Vec<IpAddr>,
    allowed_networks: Vec<IpNetwork>,
    blocked_ips: HashSet<IpAddr>,
}

impl IPAccessControl {
    pub fn is_allowed(&self, ip: IpAddr) -> bool {
        // Check if explicitly blocked
        if self.blocked_ips.contains(&ip) {
            return false;
        }

        // Check if in allowed IPs
        if self.allowed_ips.contains(&ip) {
            return true;
        }

        // Check if in allowed networks
        self.allowed_networks.iter().any(|net| net.contains(ip))
    }
}

// Configuration
[access_control]
allowed_ips = ["10.0.1.100", "10.0.1.101"]
allowed_networks = ["10.0.0.0/8", "192.168.1.0/24"]
blocked_ips = []  # Dynamically populated by rate limiter
```

#### C. **Reverse Proxy with nginx**
```nginx
# Use nginx as reverse proxy for additional security
upstream redititi {
    server 127.0.0.1:6379;
}

server {
    listen 443 ssl http2;
    server_name redititi.example.com;

    ssl_certificate /etc/nginx/ssl/cert.pem;
    ssl_certificate_key /etc/nginx/ssl/key.pem;

    # Security headers
    add_header Strict-Transport-Security "max-age=31536000" always;
    add_header X-Frame-Options "DENY" always;
    add_header X-Content-Type-Options "nosniff" always;

    # Rate limiting
    limit_req_zone $binary_remote_addr zone=redititi:10m rate=10r/s;
    limit_req zone=redititi burst=20 nodelay;

    location / {
        proxy_pass http://redititi;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
    }
}
```

**Benefits**:
- Network-level protection
- DDoS mitigation
- IP-based access control
- Additional layer of security

---

### 10. **Secure Token Management**

**Risk**: Token leakage through logs, environment variables, or process listings.

**Impact**: HIGH
**Effort**: LOW
**Priority**: 🟡 **HIGH**

**Recommendations**:

#### A. **Token Rotation**
```rust
pub struct TokenManager {
    tokens: HashMap<String, TokenMetadata>,
    rotation_interval: Duration,
}

struct TokenMetadata {
    created_at: Instant,
    expires_at: Instant,
    permissions: Vec<Permission>,
    rotation_count: u32,
}

impl TokenManager {
    pub fn rotate_token(&mut self, old_token: &str) -> Result<String> {
        // Generate new token
        let new_token = generate_secure_token();

        // Copy metadata
        let metadata = self.tokens.get(old_token).ok_or(Error::TokenNotFound)?;
        let new_metadata = TokenMetadata {
            created_at: Instant::now(),
            expires_at: Instant::now() + self.rotation_interval,
            permissions: metadata.permissions.clone(),
            rotation_count: metadata.rotation_count + 1,
        };

        // Insert new, mark old as deprecated
        self.tokens.insert(new_token.clone(), new_metadata);
        self.deprecate_token(old_token);

        Ok(new_token)
    }
}
```

#### B. **Token Storage Best Practices**
```rust
// NEVER log tokens
log::info!("Authentication successful for user: {}", user_id);  // Good
log::info!("Token: {}", token);  // BAD! Never do this!

// Use environment variables or secret management
use std::env;

fn get_token() -> Result<String> {
    // Read from environment
    env::var("REDITITI_TOKEN")
        .or_else(|_| {
            // Fallback to secure secret store
            read_from_secret_store("redititi/token")
        })
}

// Use process isolation
fn hide_token_from_process_list() {
    // Don't pass token as command-line argument
    // Bad: ./redititi --token abc123
    // Good: ./redititi (reads from env or file)
}
```

#### C. **Secure Token Generation**
```rust
use rand::RngCore;
use sha2::{Sha256, Digest};

pub fn generate_secure_token() -> String {
    let mut rng = rand::thread_rng();
    let mut bytes = [0u8; 32];
    rng.fill_bytes(&mut bytes);

    // Hash for additional entropy
    let mut hasher = Sha256::new();
    hasher.update(&bytes);
    let result = hasher.finalize();

    // Encode as hex or base64
    hex::encode(result)
}
```

**Benefits**:
- Reduces token compromise window
- Automatic security improvements
- Audit trail of token usage
- Compliance with security policies

---

## 📋 Implementation Roadmap

### Phase 1: Critical Security (1-2 weeks)
1. ✅ TLS/SSL encryption
2. ✅ Rate limiting
3. ✅ Input validation
4. ✅ Audit logging

### Phase 2: Enhanced Security (2-3 weeks)
5. ✅ JWT authentication
6. ✅ Resource quotas
7. ✅ IP access control
8. ✅ Token rotation

### Phase 3: Advanced Security (3-4 weeks)
9. ✅ Session sandboxing (namespaces, cgroups)
10. ✅ OAuth integration
11. ✅ Encrypted storage
12. ✅ mTLS support

### Phase 4: Enterprise Features (ongoing)
13. ✅ LDAP/Active Directory integration
14. ✅ SIEM integration
15. ✅ Security scanning automation
16. ✅ Compliance reporting

---

## 🔐 Security Best Practices for Deployment

### 1. **Production Checklist**
- [ ] Enable TLS with valid certificates
- [ ] Configure rate limiting
- [ ] Set up audit logging
- [ ] Configure resource quotas
- [ ] Implement IP allowlisting
- [ ] Enable firewall rules
- [ ] Use strong tokens (32+ bytes)
- [ ] Rotate tokens regularly
- [ ] Monitor logs for suspicious activity
- [ ] Keep dependencies updated
- [ ] Run security scans regularly

### 2. **Monitoring & Alerting**
```yaml
# Example Prometheus alerts
groups:
  - name: redititi_security
    rules:
      - alert: HighAuthFailureRate
        expr: rate(redititi_auth_failures_total[5m]) > 10
        annotations:
          summary: "High authentication failure rate detected"

      - alert: RateLimitExceeded
        expr: rate(redititi_rate_limit_violations_total[1m]) > 5
        annotations:
          summary: "Client exceeding rate limits"

      - alert: UnusualCommandActivity
        expr: rate(redititi_commands_total[5m]) > 1000
        annotations:
          summary: "Unusual command injection rate"
```

### 3. **Incident Response Plan**
1. **Detection**: Monitor audit logs and alerts
2. **Containment**: Block malicious IPs, revoke tokens
3. **Investigation**: Analyze logs, identify attack vector
4. **Recovery**: Restore from backups, patch vulnerabilities
5. **Post-Mortem**: Document incident, improve defenses

---

## 🎓 Security Training Resources

- [OWASP Top 10](https://owasp.org/www-project-top-ten/)
- [CIS Benchmarks](https://www.cisecurity.org/cis-benchmarks/)
- [NIST Cybersecurity Framework](https://www.nist.gov/cyberframework)
- [Rust Security Working Group](https://www.rust-lang.org/governance/wgs/wg-security)

---

## 📞 Reporting Security Vulnerabilities

If you discover a security vulnerability, please email: security@titi.dev

**Do NOT** create public GitHub issues for security vulnerabilities.

We follow responsible disclosure practices and will work with you to address issues promptly.

---

**Document Version**: 1.0
**Last Review**: 2025-12-17
**Next Review**: 2025-03-17
