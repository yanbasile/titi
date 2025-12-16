# Battle Tests User Guide

## Overview

Battle tests are comprehensive, real-world scenario tests designed to validate production readiness of the Titi terminal emulator. Unlike unit tests, these tests simulate actual usage patterns and stress conditions you'd encounter in production.

## Quick Start

```bash
# Run all battle tests
for test in battle_security battle_real_apps battle_resources battle_network battle_multi_agent battle_stability; do
  cargo test --test $test -- --ignored --nocapture
done

# Or run individually
cargo test --test battle_security -- --ignored --nocapture
```

## Why Battle Tests?

Battle tests go beyond unit tests to validate:
- **Real-world scenarios**: Actual usage patterns
- **Stress conditions**: System limits and edge cases
- **Production readiness**: Can it handle real workloads?
- **Concurrency**: Multiple agents/sessions simultaneously
- **Resilience**: Recovery from failures
- **Security**: Input validation boundaries

## Test Suite

### 1. Security & Edge Cases (`battle_security`)

**Duration**: ~50ms
**Purpose**: Validate input sanitization and security boundaries

**What it tests**:
- Malformed ANSI sequences (incomplete CSI, huge parameters)
- Invalid UTF-8 byte sequences
- Extremely long lines (>100KB)
- Binary data injection (null bytes, 0xFF)
- Control character abuse
- Nested ANSI escape sequences

**Run it**:
```bash
cargo test --test battle_security -- --ignored --nocapture
```

**Expected output**:
```
✅ Security & Edge Case Testing PASSED!
   No crashes, panics, or security vulnerabilities detected
```

**Why it matters**: Ensures the terminal can't be crashed or exploited by malicious input.

---

### 2. Real Application Integration (`battle_real_apps`)

**Duration**: ~100ms
**Purpose**: Test with actual command-line applications

**What it tests**:
- `ls --color=always` (ANSI color codes)
- `cat` (file content display)
- `grep --color=always` (search highlighting)
- `echo` with escape sequences
- Cursor positioning sequences

**Run it**:
```bash
cargo test --test battle_real_apps -- --ignored --nocapture
```

**Manual verification**: For full validation, manually test with:
- `vim` - Text editing
- `htop` - System monitoring
- `nano` - Another text editor
- `top` - Process monitoring

---

### 3. Resource Exhaustion (`battle_resources`)

**Duration**: ~5 seconds
**Purpose**: Test graceful degradation under extreme load

**What it tests**:
- Create 100+ panes
- Fill scrollback buffer to 10,000 lines
- Execute 10,000 rapid-fire commands
- Process 10KB+ ANSI sequences
- Memory pressure handling

**Run it**:
```bash
cargo test --test battle_resources -- --ignored --nocapture
```

**Performance metrics you'll see**:
- Scrollback fill rate: ~14,000 lines/sec
- Command processing: ~34,000 cmd/sec
- 100 panes created successfully

---

### 4. Network Disruption & Recovery (`battle_network`)

**Duration**: ~3-4 seconds
**Purpose**: Test server resilience under network failures

**What it tests**:
- Sudden client disconnection (mid-operation)
- Client reconnection capability
- Session persistence after disconnect
- Rapid reconnection cycles (10 iterations)

**Run it**:
```bash
cargo test --test battle_network -- --ignored --nocapture
```

**Success criteria**:
- Server continues running after disconnects
- Sessions persist across reconnections
- No data corruption
- Clean connection handling

---

### 5. Multi-Agent Orchestration (`battle_multi_agent`)

**Duration**: ~1-2 seconds
**Purpose**: Test concurrent AI agents with separate terminal sessions

**What it tests**:
- 5 concurrent agents, each with own session
- Session isolation (no data leakage)
- Pub/sub channel scalability
- Memory management with multiple sessions

**Agent workloads**:
1. **FileMonitor**: Continuous file monitoring (50 messages)
2. **BuildSystem**: Build logs (30 messages)
3. **TestRunner**: Streaming test output (40 messages)
4. **LogAggregator**: Log processing (35 messages)
5. **InteractiveShell**: Shell commands (25 messages)

**Run it**:
```bash
cargo test --test battle_multi_agent -- --ignored --nocapture
```

**Expected throughput**:
- FileMonitor: ~31 msg/sec
- BuildSystem: ~12 msg/sec
- TestRunner: ~16 msg/sec
- LogAggregator: ~14 msg/sec
- InteractiveShell: ~8 msg/sec

---

### 6. Long-Running Stability (`battle_stability`)

**Duration**: 5 minutes (300 seconds)
**Purpose**: Simulate 24-hour session with accelerated event rate

**What it tests**:
- Memory leaks over extended runtime
- Resource cleanup consistency
- Session persistence
- Performance consistency

**Operations simulated**:
- 2,900+ cycles of various operations
- 400+ command executions
- Pane creation/cleanup
- 800+ messages received
- Continuous background activity

**Run it**:
```bash
cargo test --test battle_stability -- --ignored --nocapture
```

**Progress reports every 30 seconds**:
```
📈 Progress: 30.0% | Elapsed: 90s | Cycles: 883 | Commands: 144
```

**Success criteria**:
- >1,000 cycles executed
- >100 commands executed
- Session remains functional throughout
- No memory growth detected

---

## Running All Tests

### Sequential Execution

Run tests one at a time (recommended):

```bash
#!/bin/bash
tests=(
  "battle_security"
  "battle_real_apps"
  "battle_resources"
  "battle_network"
  "battle_multi_agent"
  "battle_stability"
)

for test in "${tests[@]}"; do
  echo "Running $test..."
  cargo test --test $test -- --ignored --nocapture
  echo ""
done
```

### Parallel Execution

**Not recommended** - battle tests use specific ports and may conflict.

---

## Interpreting Results

### Success

All tests passing looks like:

```
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured
```

With test-specific success messages like:
- `✅ Security & Edge Case Testing PASSED!`
- `✅ Multi-Agent Orchestration Test PASSED!`

### Failure

Failed tests show:

```
test result: FAILED. 0 passed; 1 failed; 0 ignored
```

With detailed error messages and stack traces.

Common failure reasons:
- **Port conflicts**: Another test or process using the port
- **Timeout**: Test took too long (may need adjustment)
- **Resource limits**: System may have limits on file descriptors
- **Network issues**: Loopback interface problems

---

## Performance Benchmarking

Run the benchmark script to collect detailed metrics:

```bash
./scripts/benchmark_performance.sh
```

This generates a report in `benchmark_results/` with:
- Test execution times
- Memory usage (resident set size)
- Throughput metrics (commands/sec, lines/sec)

---

## Troubleshooting

### Test Hangs

If a test hangs:
1. Check for port conflicts: `lsof -i :19999-20100`
2. Kill stuck processes: `pkill -f battle_`
3. Wait 10 seconds before retrying

### Port Conflicts

Each test uses unique ports:
- `battle_security`: No network
- `battle_real_apps`: No network
- `battle_resources`: No network
- `battle_network`: Ports 20001-20004
- `battle_multi_agent`: Port 19999
- `battle_stability`: Port 20100

### Out of Memory

Resource exhaustion test creates 100 panes. If OOM:
1. Increase system limits: `ulimit -n 2048`
2. Close other applications
3. Edit test to reduce pane count (line 127 of `resource_exhaustion.rs`)

### Slow Tests

Tests are marked `#[ignore]` to prevent running during normal `cargo test`.

To include them:
```bash
cargo test -- --ignored --nocapture
```

---

## Continuous Integration

### GitHub Actions Example

```yaml
name: Battle Tests

on: [push, pull_request]

jobs:
  battle-tests:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v2
      - uses: actions-rs/toolchain@v1
        with:
          toolchain: stable

      - name: Run battle tests
        run: |
          for test in battle_security battle_real_apps battle_resources battle_network battle_multi_agent; do
            cargo test --test $test -- --ignored --nocapture
          done

      # Skip stability test in CI (takes 5 minutes)
```

---

## Contributing New Battle Tests

### 1. Create Test File

```rust
// tests/battle/my_new_test.rs
use tokio::runtime::Runtime;

#[test]
#[ignore]  // Important: mark as ignored
fn test_my_scenario() {
    println!("╔═══════════════════════════════════╗");
    println!("║  BATTLE TEST: My Scenario         ║");
    println!("╚═══════════════════════════════════╝");

    // Your test logic here

    println!("✅ My Scenario Test PASSED!");
}
```

### 2. Register in Cargo.toml

```toml
[[test]]
name = "battle_my_test"
path = "tests/battle/my_new_test.rs"
```

### 3. Update Documentation

Add your test to this guide with:
- Purpose
- Duration
- What it tests
- How to run it
- Success criteria

---

## FAQ

**Q: Why are battle tests marked `#[ignore]`?**
A: They take longer to run (5+ minutes total) and would slow down regular `cargo test`.

**Q: Can I run battle tests in release mode?**
A: Yes! Use `cargo test --release --test battle_security -- --ignored --nocapture`

**Q: Do battle tests require root/sudo?**
A: No, they run as regular user.

**Q: Can I run battle tests on Windows/macOS?**
A: Yes, but some tests (like network tests) may need adjustments for platform-specific behavior.

**Q: How do I add my own battle test?**
A: See "Contributing New Battle Tests" section above.

---

## Best Practices

1. **Run locally before CI**: Catch issues early
2. **Run after significant changes**: Especially to core terminal logic
3. **Monitor resource usage**: Watch memory during long tests
4. **Save benchmark results**: Track performance over time
5. **Document failures**: Note any intermittent failures

---

## Results Summary

Current battle test status:

| Test | Status | Duration | Key Metrics |
|------|--------|----------|-------------|
| Security | ✅ PASS | 50ms | 6/6 sub-tests |
| Real Apps | ✅ PASS | 100ms | 5/5 apps |
| Resources | ✅ PASS | 5s | 34K cmd/sec, 100 panes |
| Network | ✅ PASS | 3.6s | 4/4 resilience tests |
| Multi-Agent | ✅ PASS | 1.6s | 5/5 agents |
| Stability | ✅ PASS | 300s | 2,900+ cycles |

**Overall**: 6/6 tests passing (100%)

---

**Last Updated**: 2025-12-16
**Version**: 1.0
**Maintainer**: Titi Development Team
