# Implementation Roadmap: Merge → Phase 3 → Battle Test

This document outlines the execution plan for completing Titi + Redititi and making it production-ready.

## Table of Contents

1. [Step 1: Merge to Main & Release](#step-1-merge-to-main--release)
2. [Step 2: Phase 3 - Terminal Integration](#step-2-phase-3---terminal-integration)
3. [Step 3: Battle Testing](#step-3-battle-testing)
4. [Timeline](#timeline)
5. [Success Criteria](#success-criteria)

---

## Step 1: Merge to Main & Release

### Current Situation

**Branch**: `claude/gpu-terminal-emulator-01BwaHdRuJp8pDzLXVM8Ua3w`
**Status**: All work complete and pushed
**Issue**: Cannot push directly to `main` due to branch naming restrictions

### Options for Merging

#### Option A: Create Pull Request via GitHub (Recommended)

**Steps:**
1. Go to GitHub repository: `https://github.com/yanbasile/titi`
2. You'll see a banner: "claude/gpu-terminal-emulator-01BwaHdRuJp8pDzLXVM8Ua3w had recent pushes"
3. Click **"Compare & pull request"**
4. Set:
   - **Base branch**: `main` (create if doesn't exist)
   - **Compare branch**: `claude/gpu-terminal-emulator-01BwaHdRuJp8pDzLXVM8Ua3w`
5. Title: "Merge Titi Terminal + Redititi Server - Initial Release"
6. Description:
   ```markdown
   ## Summary

   Complete implementation of Titi terminal emulator and Redititi automation server.

   ## What's Included

   - ✅ GPU-accelerated terminal emulator (titi)
   - ✅ Redis-like automation server (redititi)
   - ✅ Complete documentation suite
   - ✅ 90+ unit tests
   - ✅ Battle test plan

   ## Documentation

   - [GETTING_STARTED.md](GETTING_STARTED.md) - Installation guide
   - [ARCHITECTURE.md](ARCHITECTURE.md) - System architecture
   - [AIDER_COMPATIBILITY.md](docs/AIDER_COMPATIBILITY.md) - AI pair programming
   - [BATTLE_TEST_PLAN.md](BATTLE_TEST_PLAN.md) - Testing strategy

   ## Next Steps

   - Phase 3: Terminal integration with redititi
   - Phase 4: Python client (titipy)
   - Phase 5: Battle testing
   ```
7. Click **"Create pull request"**
8. Click **"Merge pull request"**
9. Click **"Confirm merge"**

#### Option B: Set Claude Branch as Default

**Steps:**
1. Go to repository Settings → Branches
2. Change default branch to `claude/gpu-terminal-emulator-01BwaHdRuJp8pDzLXVM8Ua3w`
3. This makes it the "main" branch for all practical purposes

#### Option C: Manual Local Merge

If you have local access:
```bash
git checkout main
git merge claude/gpu-terminal-emulator-01BwaHdRuJp8pDzLXVM8Ua3w
git push origin main
```

### Release Tagging

After merge, create a release tag:

```bash
git checkout main
git pull
git tag -a v0.9.0 -m "Pre-release: Titi Terminal + Redititi Server

Features:
- GPU-accelerated terminal emulator
- Redis-like automation server
- Complete documentation
- 90+ unit tests

Status: Phase 3 (integration) pending"

git push origin v0.9.0
```

### What's Released (v0.9.0)

**Working Components:**
- ✅ `titi` - Terminal emulator (standalone)
- ✅ `redititi` - Automation server (standalone)
- ✅ All documentation
- ✅ Test infrastructure

**Not Yet Working:**
- ❌ Titi → Redititi communication
- ❌ Headless mode
- ❌ Multi-agent orchestration (requires Phase 3)

---

## Step 2: Phase 3 - Terminal Integration

### Goal

Connect Titi terminals to Redititi server to enable full automation and multi-agent orchestration.

### Timeline

**Estimated Duration**: 1-2 weeks
- Week 1: Core integration
- Week 2: Headless mode + polish

### Implementation Tasks

#### Task 1: Server Client Module (3-4 days)

**File**: `src/server_client/mod.rs`

```rust
//! Client for connecting to redititi server

use tokio::net::TcpStream;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use std::sync::Arc;
use tokio::sync::RwLock;

pub struct ServerClient {
    stream: Arc<RwLock<TcpStream>>,
    session_id: String,
    pane_id: String,
    authenticated: bool,
}

impl ServerClient {
    /// Connect to redititi server
    pub async fn connect(addr: &str) -> Result<Self, std::io::Error> {
        let stream = TcpStream::connect(addr).await?;

        Ok(Self {
            stream: Arc::new(RwLock::new(stream)),
            session_id: String::new(),
            pane_id: String::new(),
            authenticated: false,
        })
    }

    /// Authenticate with token
    pub async fn authenticate(&mut self, token: &str) -> Result<(), String> {
        self.send_command(&format!("AUTH {}", token)).await?;
        let response = self.read_response().await?;

        if response.starts_with("+OK") {
            self.authenticated = true;
            Ok(())
        } else {
            Err("Authentication failed".to_string())
        }
    }

    /// Create or join session
    pub async fn create_session(&mut self, name: Option<&str>) -> Result<String, String> {
        let cmd = if let Some(n) = name {
            format!("CREATE SESSION {}", n)
        } else {
            "CREATE SESSION".to_string()
        };

        self.send_command(&cmd).await?;
        let response = self.read_response().await?;

        if let Some(session_id) = response.strip_prefix("+OK ") {
            self.session_id = session_id.trim().to_string();
            Ok(self.session_id.clone())
        } else {
            Err("Failed to create session".to_string())
        }
    }

    /// Create pane in current session
    pub async fn create_pane(&mut self, name: Option<&str>) -> Result<String, String> {
        let cmd = if let Some(n) = name {
            format!("CREATE PANE {} {}", self.session_id, n)
        } else {
            format!("CREATE PANE {}", self.session_id)
        };

        self.send_command(&cmd).await?;
        let response = self.read_response().await?;

        if let Some(pane_id) = response.strip_prefix("+OK ") {
            self.pane_id = pane_id.trim().to_string();
            Ok(self.pane_id.clone())
        } else {
            Err("Failed to create pane".to_string())
        }
    }

    /// Subscribe to input channel
    pub async fn subscribe_input(&mut self) -> Result<(), String> {
        let channel = format!("{}/pane-{}/input", self.session_id, self.pane_id);
        self.send_command(&format!("SUBSCRIBE {}", channel)).await?;
        let response = self.read_response().await?;

        if response.starts_with("+OK") {
            Ok(())
        } else {
            Err("Failed to subscribe to input".to_string())
        }
    }

    /// Publish output to channel
    pub async fn publish_output(&self, data: &str) -> Result<(), String> {
        let channel = format!("{}/pane-{}/output", self.session_id, self.pane_id);
        let cmd = format!("PUBLISH {} {}", channel, data);
        self.send_command(&cmd).await?;
        Ok(())
    }

    /// Read message from input channel (non-blocking)
    pub async fn read_input(&mut self) -> Result<Option<String>, String> {
        let channel = format!("{}/pane-{}/input", self.session_id, self.pane_id);
        self.send_command(&format!("RPOP {}", channel)).await?;
        let response = self.read_response().await?;

        if response.starts_with("-ERR") {
            Ok(None)  // Queue empty
        } else if let Some(msg) = response.strip_prefix("\"").and_then(|s| s.strip_suffix("\"")) {
            Ok(Some(msg.to_string()))
        } else {
            Ok(None)
        }
    }

    // Helper methods
    async fn send_command(&self, cmd: &str) -> Result<(), String> {
        let mut stream = self.stream.write().await;
        stream.write_all(cmd.as_bytes()).await
            .map_err(|e| e.to_string())?;
        stream.write_all(b"\n").await
            .map_err(|e| e.to_string())?;
        Ok(())
    }

    async fn read_response(&self) -> Result<String, String> {
        let stream = self.stream.read().await;
        let mut reader = BufReader::new(&*stream);
        let mut line = String::new();
        reader.read_line(&mut line).await
            .map_err(|e| e.to_string())?;
        Ok(line.trim().to_string())
    }
}
```

**Tests**: `src/server_client/mod.rs`

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_connection() {
        // Start test redititi server
        // Connect client
        // Verify connection works
    }

    #[tokio::test]
    async fn test_authentication() {
        // Test valid token
        // Test invalid token
        // Test max attempts
    }

    #[tokio::test]
    async fn test_session_lifecycle() {
        // Create session
        // Create pane
        // Verify IDs returned
    }
}
```

#### Task 2: Terminal Integration (2-3 days)

**File**: `src/terminal/mod.rs` (modifications)

```rust
use crate::server_client::ServerClient;

pub struct Terminal {
    // Existing fields...
    pty: PtyPair,
    grid: Arc<Mutex<Grid>>,
    parser: TerminalParser,

    // New fields
    server_client: Option<Arc<RwLock<ServerClient>>>,
    publish_output: bool,
}

impl Terminal {
    pub fn new_with_server(
        cols: usize,
        rows: usize,
        server_client: ServerClient,
    ) -> Self {
        // Create terminal with server integration
        let mut terminal = Self::new(cols, rows);
        terminal.server_client = Some(Arc::new(RwLock::new(server_client)));
        terminal.publish_output = true;
        terminal
    }

    pub async fn start_server_integration(&mut self) {
        if let Some(client) = &self.server_client {
            // Start background task for input polling
            let client_clone = client.clone();
            let pty_clone = self.pty.clone();

            tokio::spawn(async move {
                loop {
                    let mut client = client_clone.write().await;

                    // Poll for input commands
                    if let Ok(Some(cmd)) = client.read_input().await {
                        // Write to PTY
                        let mut pty = pty_clone.master.lock().unwrap();
                        let _ = pty.write_all(cmd.as_bytes());
                        let _ = pty.write_all(b"\n");
                    }

                    // Sleep briefly to avoid busy loop
                    tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
                }
            });
        }
    }

    pub async fn publish_output_if_needed(&self) {
        if self.publish_output {
            if let Some(client) = &self.server_client {
                // Get dirty lines from grid
                let dirty_lines = {
                    let mut grid = self.grid.lock().unwrap();
                    grid.get_dirty_lines()
                };

                // Publish each dirty line
                let client = client.read().await;
                for (line_num, content) in dirty_lines {
                    let output = format!("L{}: {}", line_num, content);
                    let _ = client.publish_output(&output).await;
                }
            }
        }
    }
}
```

**File**: `src/main.rs` (modifications)

```rust
use titi::server_client::ServerClient;

#[tokio::main]
async fn main() {
    // Parse command line args
    let args: Vec<String> = std::env::args().collect();
    let mut server_addr: Option<String> = None;
    let mut session_name: Option<String> = None;
    let mut headless = false;

    // Parse --server, --session, --headless flags
    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--server" => {
                server_addr = Some(args[i + 1].clone());
                i += 2;
            }
            "--session" => {
                session_name = Some(args[i + 1].clone());
                i += 2;
            }
            "--headless" => {
                headless = true;
                i += 1;
            }
            _ => i += 1,
        }
    }

    // If server specified, connect
    let server_client = if let Some(addr) = server_addr {
        let mut client = ServerClient::connect(&addr).await.unwrap();

        // Read token from ~/.titi/token
        let token = std::fs::read_to_string(
            dirs::home_dir().unwrap().join(".titi/token")
        ).unwrap();

        client.authenticate(&token).await.unwrap();
        client.create_session(session_name.as_deref()).await.unwrap();
        client.create_pane(None).await.unwrap();
        client.subscribe_input().await.unwrap();

        Some(client)
    } else {
        None
    };

    if headless {
        run_headless(server_client).await;
    } else {
        run_windowed(server_client).await;
    }
}
```

#### Task 3: Headless Mode (2-3 days)

**File**: `src/headless.rs` (new)

```rust
//! Headless mode - run terminal without GPU

use crate::terminal::Terminal;
use crate::server_client::ServerClient;
use std::time::Duration;
use tokio::time;

pub async fn run_headless(server_client: Option<ServerClient>) {
    let mut terminal = if let Some(client) = server_client {
        Terminal::new_with_server(80, 24, client)
    } else {
        panic!("Headless mode requires --server flag");
    };

    // Start server integration
    terminal.start_server_integration().await;

    // Main loop - just process PTY I/O
    loop {
        // Read from PTY
        if let Some(output) = terminal.read_pty_output() {
            terminal.process_output(&output);
            terminal.publish_output_if_needed().await;
        }

        // Small sleep to avoid busy loop
        time::sleep(Duration::from_millis(10)).await;
    }
}
```

**Update**: `src/main.rs`

```rust
mod headless;

async fn run_headless(server_client: Option<ServerClient>) {
    headless::run_headless(server_client).await;
}

async fn run_windowed(server_client: Option<ServerClient>) {
    // Existing GPU-based terminal code
    // Now optionally with server integration
}
```

#### Task 4: Configuration & CLI (1 day)

**Update**: `src/config.rs`

```rust
#[derive(Debug, Clone)]
pub struct Config {
    // Existing fields...
    pub font: FontConfig,
    pub window: WindowConfig,
    pub shell: ShellConfig,
    pub colors: ColorConfig,

    // New server fields
    pub server: Option<ServerConfig>,
}

#[derive(Debug, Clone)]
pub struct ServerConfig {
    pub address: String,  // Default: "127.0.0.1:6379"
    pub token_file: PathBuf,  // Default: "~/.titi/token"
    pub auto_connect: bool,  // Default: false
    pub session_name: Option<String>,
}
```

**CLI Arguments**:

```bash
# Connect to server
titi --server 127.0.0.1:6379 --session my-session

# Headless mode
titi --headless --server 127.0.0.1:6379

# With custom token
titi --server 127.0.0.1:6379 --token-file /path/to/token

# Standalone (no server)
titi
```

#### Task 5: Integration Tests (1-2 days)

**File**: `tests/integration/server_integration.rs`

```rust
#[tokio::test]
async fn test_command_injection_e2e() {
    // 1. Start redititi server
    // 2. Start titi in headless mode
    // 3. Send INJECT command via redititi
    // 4. Verify command executed in terminal
    // 5. Capture output via RPOP
}

#[tokio::test]
async fn test_screen_capture_e2e() {
    // 1. Start redititi + titi
    // 2. Execute command in terminal
    // 3. Read output from channel
    // 4. Verify output captured correctly
}

#[tokio::test]
async fn test_multi_terminal_coordination() {
    // 1. Start 10 titi terminals
    // 2. Send commands to each via INJECT
    // 3. Verify all execute correctly
    // 4. Capture all outputs
}
```

### Deliverables (Phase 3)

- ✅ Server client module
- ✅ Terminal integration
- ✅ Headless mode
- ✅ CLI arguments
- ✅ Integration tests
- ✅ Documentation update

### Testing Phase 3

```bash
# Terminal 1: Start redititi
cargo run --bin redititi --release

# Save the token shown on startup

# Terminal 2: Start titi with server
cargo run --release -- --server 127.0.0.1:6379 --session test-session

# Terminal 3: Test command injection
nc localhost 6379
AUTH <your-token>
PUBLISH session-test-session/pane-1/input "ls -la"

# Verify: Terminal 2 should execute 'ls -la'

# Capture output
SUBSCRIBE session-test-session/pane-1/output
RPOP session-test-session/pane-1/output
```

---

## Step 3: Battle Testing

### Goal

Stress test the complete Titi + Redititi system to ensure production readiness.

### Timeline

**Estimated Duration**: 5 weeks (as per BATTLE_TEST_PLAN.md)
- Week 1: Titi stress tests
- Week 2: Redititi stress tests
- Week 3: Integration stress tests
- Week 4: Chaos engineering
- Week 5: Polish and fixes

### Priority Test Suites

#### Week 1: Titi Terminal Stress Tests

**Priority 1 (Must Pass):**

1. **Extreme Output Stress**
   ```bash
   cargo test --test extreme_output sustained_100k_lines
   cargo test --test extreme_output burst_1m_lines
   ```
   - Target: 100k lines/sec sustained
   - Target: 1M line burst without crash

2. **Pane Management Stress**
   ```bash
   cargo test --test pane_stress create_destroy_1000
   cargo test --test pane_stress 200_simultaneous_panes
   ```
   - Target: 1000 create/destroy cycles
   - Target: 200 panes active simultaneously

3. **Resource Exhaustion**
   ```bash
   cargo test --test resource_exhaustion memory_limits
   cargo test --test resource_exhaustion fd_limits
   ```
   - Target: Graceful handling when limits hit

**Priority 2 (Should Pass):**

4. **Rendering Stress**
5. **Input Stress**

**Priority 3 (Nice to Have):**

6. **24h Soak Test**
   ```bash
   cargo test --test soak_tests test_24_hour_uptime -- --ignored --nocapture
   ```

#### Week 2: Redititi Server Stress Tests

**Priority 1 (Must Pass):**

1. **Connection Stress**
   ```bash
   cargo test --test connection_stress test_1000_concurrent_clients
   ```
   - Target: 1000 concurrent clients

2. **Command Processing**
   ```bash
   cargo test --test command_stress test_10k_commands_per_second
   ```
   - Target: 10k commands/sec throughput

3. **Channel Stress**
   ```bash
   cargo test --test channel_stress test_10k_active_channels
   cargo test --test channel_stress test_queue_overflow
   ```

#### Week 3: Integration Stress Tests (CRITICAL)

**This is where we test Titi + Redititi together!**

1. **Command Injection Stress**
   ```bash
   cargo test --test titi_redititi_stress test_1000_commands_to_100_terminals
   ```
   - 100 Titi terminals (headless)
   - 1000 commands sent to each
   - Verify all executed correctly

2. **Output Streaming Stress**
   ```bash
   cargo test --test titi_redititi_stress test_high_output_streaming
   ```
   - 50 terminals outputting 10k lines/sec each
   - All publishing to channels
   - Verify no message loss

3. **Multi-Agent Coordination**
   ```bash
   cargo test --test multi_agent_stress test_100_parallel_agents
   ```
   - Simulate 100 AI agents
   - All coordinating via redititi
   - Verify correct execution

#### Week 4: Chaos Engineering

1. **Failure Injection**
   ```bash
   cargo test --test failure_injection test_terminal_process_kill
   cargo test --test failure_injection test_server_restart
   ```
   - Random terminal kills
   - Server restarts
   - Verify recovery (MTTR <5sec)

2. **Resource Limits**
   ```bash
   cargo test --test resource_limits test_disk_full
   cargo test --test resource_limits test_oom
   ```

#### Week 5: Polish & Validation

- Fix all issues found
- Re-run failed tests
- Performance tuning
- Documentation updates

### Success Criteria

**All tests from BATTLE_TEST_PLAN.md must pass:**

- ✅ Titi: 100k lines/sec, 100 panes, 7-day uptime
- ✅ Redititi: 1000 clients, 10k commands/sec, 99.9% uptime
- ✅ Integration: 100+ terminals, zero message loss, <5sec MTTR
- ✅ Test coverage: >80%
- ✅ Zero critical bugs
- ✅ Performance benchmarks met

### Continuous Testing

**GitHub Actions**: `.github/workflows/battle-tests.yml`

```yaml
name: Battle Tests

on: [push, pull_request]

jobs:
  unit-tests:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v2
      - run: cargo test --lib

  stress-tests:
    runs-on: ubuntu-latest
    timeout-minutes: 30
    steps:
      - uses: actions/checkout@v2
      - run: cargo test --test stress

  integration-tests:
    runs-on: ubuntu-latest
    timeout-minutes: 20
    steps:
      - uses: actions/checkout@v2
      - run: cargo test --test integration

  soak-tests:
    runs-on: ubuntu-latest
    timeout-minutes: 1440  # 24 hours
    if: github.ref == 'refs/heads/main'
    steps:
      - uses: actions/checkout@v2
      - run: cargo test --test soak_tests -- --ignored --nocapture
```

---

## Timeline

### Overall Schedule

```
Week 0: Merge to Main & Release v0.9.0
  └─ Create PR, merge, tag release

Week 1-2: Phase 3 Implementation
  ├─ Week 1: Server client + terminal integration
  └─ Week 2: Headless mode + CLI + tests

Week 3-7: Battle Testing (5 weeks)
  ├─ Week 3: Titi stress tests
  ├─ Week 4: Redititi stress tests
  ├─ Week 5: Integration tests (Titi + Redititi)
  ├─ Week 6: Chaos engineering
  └─ Week 7: Polish & validation

Week 8: Release v1.0.0
  └─ Production-ready release
```

### Milestones

- **Week 0**: v0.9.0 - Pre-release (current state)
- **Week 2**: v0.9.5 - Phase 3 complete
- **Week 7**: v0.9.9 - Battle tested
- **Week 8**: v1.0.0 - Production release

---

## Success Criteria

### v0.9.0 (Merge to Main)

- ✅ All code merged to main branch
- ✅ Tagged as v0.9.0
- ✅ Documentation complete
- ✅ Builds successfully

### v0.9.5 (Phase 3 Complete)

- ✅ Titi connects to redititi
- ✅ Command injection works
- ✅ Screen capture works
- ✅ Headless mode functional
- ✅ Integration tests pass

### v1.0.0 (Production Ready)

- ✅ All battle tests pass
- ✅ 99.9% uptime demonstrated
- ✅ Performance benchmarks met
- ✅ Zero critical bugs
- ✅ Full documentation
- ✅ Ready for production use

---

## Daily Progress Tracking

### Week 1 (Phase 3 - Part 1)

**Day 1**: Server client module
- [ ] Implement connect/authenticate
- [ ] Implement session/pane creation
- [ ] Implement subscribe/publish
- [ ] Unit tests

**Day 2**: Server client module (cont.)
- [ ] Implement read_input/publish_output
- [ ] Error handling
- [ ] More unit tests
- [ ] Code review

**Day 3**: Terminal integration
- [ ] Add ServerClient to Terminal struct
- [ ] Implement output publishing
- [ ] Background task for input polling
- [ ] Basic testing

**Day 4**: Terminal integration (cont.)
- [ ] Dirty line tracking integration
- [ ] Error handling
- [ ] Integration tests
- [ ] Documentation

### Week 2 (Phase 3 - Part 2)

**Day 5**: Headless mode
- [ ] Create headless.rs module
- [ ] Implement run_headless()
- [ ] PTY I/O loop
- [ ] Testing

**Day 6**: CLI integration
- [ ] Add --server, --session flags
- [ ] Add --headless flag
- [ ] Config file updates
- [ ] Help text

**Day 7**: Integration testing
- [ ] E2E command injection test
- [ ] E2E screen capture test
- [ ] Multi-terminal test
- [ ] Bug fixes

**Day 8**: Polish & documentation
- [ ] Update GETTING_STARTED.md
- [ ] Update ARCHITECTURE.md
- [ ] Add examples
- [ ] Code cleanup

**Day 9-10**: Buffer for unexpected issues

---

## What Happens After

### Phase 4: Python Client (Optional)

After Phase 3 and battle testing, optionally create `titipy`:

```python
from titipy import TitiClient

client = TitiClient()
session = client.create_session("dev")
pane = session.create_pane()
pane.inject("ls -la")
output = pane.capture_lines(10)
```

### Phase 5: Production Deployment

- Docker images
- Kubernetes manifests
- Production documentation
- User onboarding guide

---

## Summary

**Step 1 (Now)**: Merge to main via GitHub PR → v0.9.0
**Step 2 (1-2 weeks)**: Implement Phase 3 → v0.9.5
**Step 3 (5 weeks)**: Battle test everything → v1.0.0

**End Result**: Production-ready Titi + Redititi system capable of orchestrating 100+ AI agents with 99.9% reliability! 🚀
