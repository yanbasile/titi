# Battle Test Plan: Production-Ready Titi + Redititi

This document outlines a comprehensive battle testing strategy to make Titi and Redititi production-ready and mature.

## Table of Contents

1. [Overview](#overview)
2. [Current Test Coverage](#current-test-coverage)
3. [Battle Test Strategy](#battle-test-strategy)
4. [Titi Terminal Stress Tests](#titi-terminal-stress-tests) (Suites 1-6)
5. [Redititi Server Stress Tests](#redititi-server-stress-tests) (Suites 7-11)
6. [Integration Stress Tests](#integration-stress-tests) (Suites 12-20) **★ NEW: 40+ Redititi tests**
7. [Chaos Engineering Tests](#chaos-engineering-tests) (Suites 21-22)
8. [Performance Benchmarks](#performance-benchmarks) (Suites 23-24)
9. [Security Tests](#security-tests) (Suite 25)
10. [Real-World Scenario Tests](#real-world-scenario-tests) (Suite 26)
11. [Implementation Phases](#implementation-phases)
12. [Success Criteria](#success-criteria)

**Total: 26 Test Suites | 150+ Individual Tests | Emphasis on Titi + Redititi Integration**

---

## Overview

### Goals

- **Production-Ready**: Ensure Titi and Redititi can handle production workloads
- **Mature**: Identify and fix edge cases, race conditions, and failure modes
- **Reliable**: 99.9% uptime under normal conditions
- **Scalable**: Support 100+ concurrent sessions without degradation
- **Resilient**: Graceful degradation under extreme load

### Testing Philosophy

> "If it can break, it will break. Let's break it first."

We'll use:
- **Stress Testing**: Push beyond normal limits
- **Chaos Engineering**: Inject faults deliberately
- **Property-Based Testing**: Generate random scenarios
- **Fuzz Testing**: Invalid/malformed inputs
- **Load Testing**: Sustained high usage
- **Soak Testing**: Long-running stability

---

## Current Test Coverage

### ✅ Existing Tests (90+ tests)

**Titi Terminal:**
- ✅ Grid operations (unit tests)
- ✅ ANSI parser (unit tests)
- ✅ Layout system (unit tests)
- ✅ Performance stress (10k lines/sec)
- ✅ Concurrency stress (50+ panes)
- ✅ Memory leak detection

**Redititi Server:**
- ✅ Token authentication (unit tests)
- ✅ Protocol parsing (unit tests)
- ✅ Registry operations (unit tests)
- ✅ Channel pub/sub (unit tests)
- ✅ Command handlers (unit tests)

### ❌ Missing Coverage

**Titi Terminal:**
- ❌ Extended soak tests (24+ hours)
- ❌ GPU rendering stress
- ❌ Copy/paste stress
- ❌ Mouse event flood
- ❌ Extreme resize scenarios
- ❌ Resource exhaustion

**Redititi Server:**
- ❌ Concurrent client stress (1000+ clients)
- ❌ Message queue overflow
- ❌ Channel saturation
- ❌ Authentication brute force
- ❌ Network failure handling
- ❌ Memory limits

**Integration:**
- ❌ Titi ↔ Redititi stress
- ❌ Multi-agent coordination
- ❌ Failure cascade scenarios
- ❌ Recovery from crashes

---

## Battle Test Strategy

### Phase 1: Titi Terminal Hardening (Week 1-2)

Focus on terminal emulator edge cases and limits.

**Target Metrics:**
- Handle 100k lines/sec output
- Support 100+ simultaneous panes
- Run 7+ days without memory leak
- Handle 10k resize operations/sec
- Process 1M ANSI sequences without error

### Phase 2: Redititi Server Hardening (Week 2-3)

Focus on server scalability and resilience.

**Target Metrics:**
- Handle 1000+ concurrent clients
- Process 10k commands/sec
- Support 10k active channels
- Queue 1M messages without overflow
- Recover from network failures <1sec

### Phase 3: Integration Hardening (Week 3-4)

Focus on Titi + Redititi working together.

**Target Metrics:**
- Coordinate 100+ terminals
- Handle 1000 commands/sec across all terminals
- Recover from server restart <2sec
- Support 10+ hours continuous operation
- Zero message loss under normal conditions

### Phase 4: Chaos & Production Testing (Week 4-5)

Inject faults and run real-world scenarios.

**Target Metrics:**
- Survive random process kills
- Handle disk full / out of memory
- Recover from network partitions
- Graceful degradation under extreme load
- Mean Time To Recovery (MTTR) < 5sec

---

## Titi Terminal Stress Tests

### Test Suite 1: Extreme Output Stress

**File**: `tests/stress/extreme_output.rs`

#### Test 1.1: Sustained High Throughput
```rust
#[test]
fn test_sustained_100k_lines_per_second() {
    // Generate 100k lines/sec for 60 seconds
    // Verify:
    // - No dropped output
    // - Memory stays bounded
    // - Rendering stays smooth (>30 FPS)
    // - No panic or crash
}
```

#### Test 1.2: Burst Output
```rust
#[test]
fn test_burst_1m_lines_immediate() {
    // Output 1 million lines instantly
    // Verify:
    // - Grid handles overflow correctly
    // - Scrollback works
    // - No memory explosion
    // - Terminal remains responsive
}
```

#### Test 1.3: Binary Output
```rust
#[test]
fn test_binary_data_resilience() {
    // Send random binary data (not valid UTF-8)
    // Verify:
    // - Parser doesn't panic
    // - Invalid UTF-8 handled gracefully
    // - Terminal stays functional
}
```

### Test Suite 2: Rendering Stress

**File**: `tests/stress/rendering_stress.rs`

#### Test 2.1: Rapid Screen Changes
```rust
#[test]
fn test_full_screen_redraws_60fps() {
    // Redraw entire 80x24 screen 60 times/sec for 5 min
    // Verify:
    // - GPU doesn't run out of memory
    // - FPS stays >30
    // - No visual artifacts
    // - Glyph atlas doesn't overflow
}
```

#### Test 2.2: Color Explosion
```rust
#[test]
fn test_all_16m_colors_cycling() {
    // Cycle through all RGB colors
    // Verify:
    // - Color rendering correct
    // - No shader crashes
    // - Performance acceptable
}
```

#### Test 2.3: Unicode Stress
```rust
#[test]
fn test_complex_unicode_rendering() {
    // Render emojis, CJK, RTL text, combining marks
    // Verify:
    // - Correct glyph rendering
    // - Proper width calculation
    // - No glyph atlas overflow
    // - Font fallback works
}
```

### Test Suite 3: Pane Management Stress

**File**: `tests/stress/pane_stress.rs`

#### Test 3.1: Pane Creation/Destruction Cycling
```rust
#[test]
fn test_create_destroy_1000_panes() {
    // Create and destroy 1000 panes rapidly
    // Verify:
    // - No memory leak
    // - No zombie processes
    // - Layout stays consistent
    // - Focus tracking works
}
```

#### Test 3.2: Extreme Pane Count
```rust
#[test]
fn test_200_simultaneous_panes() {
    // Create 200 panes all running commands
    // Verify:
    // - All panes functional
    // - Switching works
    // - Memory usage reasonable
    // - CPU usage acceptable
}
```

#### Test 3.3: Nested Splits Stress
```rust
#[test]
fn test_deeply_nested_splits() {
    // Create 10+ levels of nested splits
    // Verify:
    // - Layout calculation correct
    // - Rendering bounds accurate
    // - No stack overflow
    // - Pane removal works
}
```

### Test Suite 4: Input Stress

**File**: `tests/stress/input_stress.rs`

#### Test 4.1: Keyboard Event Flood
```rust
#[test]
fn test_10k_keystrokes_per_second() {
    // Send 10k keyboard events/sec
    // Verify:
    // - All input processed
    // - No input dropped
    // - Terminal responsive
    // - PTY buffers don't overflow
}
```

#### Test 4.2: Mouse Event Flood
```rust
#[test]
fn test_mouse_movement_flood() {
    // Send 1000 mouse events/sec
    // Verify:
    // - Events processed correctly
    // - No lag accumulation
    // - Focus changes work
    // - Click detection accurate
}
```

#### Test 4.3: Copy/Paste Stress
```rust
#[test]
fn test_copy_paste_large_buffers() {
    // Copy/paste 10MB of text
    // Verify:
    // - Clipboard handles large data
    // - Paste doesn't freeze terminal
    // - Memory usage reasonable
    // - No data corruption
}
```

### Test Suite 5: Resource Exhaustion

**File**: `tests/stress/resource_exhaustion.rs`

#### Test 5.1: Memory Limits
```rust
#[test]
fn test_memory_limit_behavior() {
    // Fill scrollback until near memory limit
    // Verify:
    // - Graceful buffer eviction
    // - No out-of-memory crash
    // - Warning messages logged
    // - Terminal stays functional
}
```

#### Test 5.2: File Descriptor Limits
```rust
#[test]
fn test_fd_exhaustion_handling() {
    // Create panes until FD limit
    // Verify:
    // - Clear error message
    // - Existing panes still work
    // - Can close panes and create new
    // - No leaked FDs
}
```

#### Test 5.3: GPU Resource Limits
```rust
#[test]
fn test_gpu_memory_limits() {
    // Create enough panes to stress GPU memory
    // Verify:
    // - Graceful handling of GPU OOM
    // - Fallback to software rendering?
    // - Clear error messages
    // - No driver crash
}
```

### Test Suite 6: Soak Tests

**File**: `tests/stress/soak_tests.rs`

#### Test 6.1: 24 Hour Continuous Operation
```rust
#[test]
#[ignore]
fn test_24_hour_uptime() {
    // Run terminal for 24 hours with:
    // - Periodic output
    // - Random pane operations
    // - Memory monitoring
    // Verify:
    // - No memory leak (<10% growth)
    // - No performance degradation
    // - All features still work
    // - No file descriptor leak
}
```

#### Test 6.2: Week-Long Stability
```rust
#[test]
#[ignore]
fn test_7_day_uptime() {
    // Run for 7 days (CI only)
    // Verify:
    // - Stable memory usage
    // - No resource leaks
    // - Consistent performance
}
```

---

## Redititi Server Stress Tests

### Test Suite 7: Connection Stress

**File**: `tests/server/connection_stress.rs`

#### Test 7.1: Concurrent Client Connections
```rust
#[tokio::test]
async fn test_1000_concurrent_clients() {
    // Connect 1000 clients simultaneously
    // Verify:
    // - All connections accepted
    // - Authentication works
    // - Memory usage reasonable
    // - Response time <100ms
}
```

#### Test 7.2: Rapid Connect/Disconnect
```rust
#[tokio::test]
async fn test_connection_churn() {
    // 100 clients connecting/disconnecting rapidly
    // Verify:
    // - No connection leak
    // - Cleanup happens correctly
    // - Server stays responsive
    // - No panic on disconnect
}
```

#### Test 7.3: Slow Clients
```rust
#[tokio::test]
async fn test_slow_reading_clients() {
    // Clients that read very slowly
    // Verify:
    // - Server doesn't block
    // - Backpressure handling
    // - Timeout enforcement
    // - Other clients unaffected
}
```

### Test Suite 8: Command Processing Stress

**File**: `tests/server/command_stress.rs`

#### Test 8.1: Command Throughput
```rust
#[tokio::test]
async fn test_10k_commands_per_second() {
    // Send 10k commands/sec
    // Verify:
    // - All commands processed
    // - Correct responses
    // - Latency <10ms p99
    // - No command dropped
}
```

#### Test 8.2: Complex Command Patterns
```rust
#[tokio::test]
async fn test_mixed_command_workload() {
    // Mix of:
    // - Session creation (10%)
    // - Publish (40%)
    // - Subscribe (20%)
    // - RPOP (30%)
    // Verify:
    // - All commands work correctly
    // - No deadlocks
    // - Fair scheduling
}
```

#### Test 8.3: Malformed Commands
```rust
#[tokio::test]
async fn test_invalid_command_flood() {
    // Send 10k invalid commands
    // Verify:
    // - Proper error responses
    // - Server doesn't crash
    // - Connection stays open
    // - Performance unaffected
}
```

### Test Suite 9: Channel Stress

**File**: `tests/server/channel_stress.rs`

#### Test 9.1: Channel Saturation
```rust
#[tokio::test]
async fn test_10k_active_channels() {
    // Create 10k channels
    // Verify:
    // - All channels work
    // - Memory usage reasonable
    // - Publish/subscribe works
    // - No channel leak
}
```

#### Test 9.2: Message Queue Overflow
```rust
#[tokio::test]
async fn test_queue_overflow_handling() {
    // Publish 1M messages to single channel
    // No subscribers
    // Verify:
    // - Queue has max size limit
    // - Old messages evicted
    // - Memory bounded
    // - Clear warnings logged
}
```

#### Test 9.3: Subscribe/Unsubscribe Churn
```rust
#[tokio::test]
async fn test_subscription_churn() {
    // Rapidly subscribe/unsubscribe
    // Verify:
    // - No memory leak
    // - Channel cleanup works
    // - No dangling subscriptions
}
```

### Test Suite 10: Registry Stress

**File**: `tests/server/registry_stress.rs`

#### Test 10.1: Session/Pane Creation Stress
```rust
#[tokio::test]
async fn test_1000_sessions_10k_panes() {
    // Create 1000 sessions, 10 panes each
    // Verify:
    // - Name generation works
    // - No collisions
    // - Lookup performance good
    // - Memory usage reasonable
}
```

#### Test 10.2: Registry Churn
```rust
#[tokio::test]
async fn test_create_destroy_sessions() {
    // Create/destroy sessions rapidly
    // Verify:
    // - No memory leak
    // - Panes cleaned up
    // - Registry stays consistent
}
```

### Test Suite 11: Authentication Stress

**File**: `tests/server/auth_stress.rs`

#### Test 11.1: Brute Force Protection
```rust
#[tokio::test]
async fn test_failed_auth_handling() {
    // Try 1000 wrong passwords
    // Verify:
    // - Connection closed after 3 attempts
    // - No timing attacks possible
    // - Rate limiting works
    // - Logging happens
}
```

#### Test 11.2: Token Rotation
```rust
#[tokio::test]
async fn test_token_rotation_under_load() {
    // Rotate token while clients connected
    // Verify:
    // - New clients use new token
    // - Old clients get rejected
    // - Graceful migration possible
}
```

---

## Integration Stress Tests

### Test Suite 12: Titi ↔ Redititi Integration

**File**: `tests/integration/titi_redititi_stress.rs`

#### Test 12.1: Command Injection Stress
```rust
#[tokio::test]
async fn test_1000_commands_to_100_terminals() {
    // 100 Titi terminals connected to redititi
    // Send 1000 commands to each
    // Verify:
    // - All commands executed
    // - Correct terminal receives command
    // - No command loss
    // - Output captured correctly
}
```

#### Test 12.2: Output Streaming Stress
```rust
#[tokio::test]
async fn test_high_output_streaming() {
    // 50 terminals outputting 10k lines/sec each
    // All publishing to channels
    // Verify:
    // - All output captured
    // - Channel queues don't overflow
    // - Subscribers receive correct data
    // - Performance acceptable
}
```

#### Test 12.3: Session Lifecycle Stress
```rust
#[tokio::test]
async fn test_session_create_destroy_cycle() {
    // Create/destroy 1000 sessions
    // Each with 10 panes
    // Verify:
    // - Terminals connect/disconnect correctly
    // - Channels cleaned up
    // - No orphaned terminals
    // - Registry stays consistent
}
```

### Test Suite 13: Multi-Agent Coordination

**File**: `tests/integration/multi_agent_stress.rs`

#### Test 13.1: Parallel Agent Execution
```rust
#[tokio::test]
async fn test_100_parallel_agents() {
    // Simulate 100 Aider/Claude agents
    // All executing tasks in parallel
    // Verify:
    // - All agents get resources
    // - No starvation
    // - Correct task execution
    // - Results collected properly
}
```

#### Test 13.2: Agent Communication Stress
```rust
#[tokio::test]
async fn test_agent_to_agent_messaging() {
    // Agents sending messages to each other
    // Via pub/sub channels
    // Verify:
    // - Messages delivered correctly
    // - No message loss
    // - Ordering maintained per channel
    // - No deadlocks
}
```

#### Test 13.3: Master-Worker Pattern
```rust
#[tokio::test]
async fn test_master_worker_coordination() {
    // 1 master agent coordinating 50 workers
    // Via redititi pub/sub
    // Verify:
    // - Task distribution works
    // - Results aggregated correctly
    // - Worker failure detected
    // - Work redistribution happens
}
```

#### Test 13.4: Fan-Out/Fan-In Pattern
```rust
#[tokio::test]
async fn test_fanout_fanin_pattern() {
    // Broadcast command to 100 terminals
    // Collect all outputs
    // Verify:
    // - All terminals receive command
    // - All outputs captured
    // - Timing coordinated properly
    // - Results aggregated correctly
}
```

### Test Suite 14: Redititi Protocol Stress

**File**: `tests/integration/protocol_stress.rs`

#### Test 14.1: Malformed Command Handling
```rust
#[tokio::test]
async fn test_malformed_commands() {
    // Send invalid protocol commands
    // Verify:
    // - Server doesn't crash
    // - Error responses returned
    // - Connection stays alive
    // - Other clients unaffected
}
```

#### Test 14.2: Protocol Fuzzing
```rust
#[tokio::test]
async fn test_protocol_fuzzing() {
    // Send random bytes as commands
    // Verify:
    // - Parser doesn't panic
    // - Memory usage bounded
    // - Server stays responsive
    // - Clear error logging
}
```

#### Test 14.3: Large Command Payloads
```rust
#[tokio::test]
async fn test_large_command_payloads() {
    // Send commands with 10MB payloads
    // Verify:
    // - Size limits enforced
    // - Clear error messages
    // - No memory explosion
    // - Connection properly closed
}
```

#### Test 14.4: Rapid Command Fire
```rust
#[tokio::test]
async fn test_rapid_command_fire() {
    // Send 10k commands without waiting for responses
    // Verify:
    // - Server processes all commands
    // - Responses in correct order
    // - No command dropped
    // - Backpressure works
}
```

### Test Suite 15: Channel Pub/Sub Stress

**File**: `tests/integration/pubsub_stress.rs`

#### Test 15.1: Many-to-Many Communication
```rust
#[tokio::test]
async fn test_100_publishers_100_subscribers() {
    // 100 publishers, 100 subscribers, 1 channel
    // Verify:
    // - All messages delivered
    // - Fair distribution
    // - No message duplication
    // - Performance acceptable
}
```

#### Test 15.2: Channel Saturation
```rust
#[tokio::test]
async fn test_channel_saturation() {
    // Publish faster than subscribers can consume
    // Verify:
    // - Queue depth limits enforced
    // - Slow subscriber handling
    // - Fast subscribers not blocked
    // - Backpressure applied
}
```

#### Test 15.3: Subscriber Churn
```rust
#[tokio::test]
async fn test_subscriber_churn_under_load() {
    // Constantly subscribe/unsubscribe while messaging
    // Verify:
    // - No missed messages
    // - Clean subscription state
    // - No memory leaks
    // - Channel cleanup works
}
```

#### Test 15.4: Cross-Channel Messaging
```rust
#[tokio::test]
async fn test_1000_channels_concurrent() {
    // 1000 channels, each with 10 pub/sub
    // Verify:
    // - Channel isolation maintained
    // - No cross-channel leaks
    // - Performance scales linearly
    // - Memory usage reasonable
}
```

### Test Suite 16: Headless Terminal Integration

**File**: `tests/integration/headless_stress.rs`

#### Test 16.1: Headless Lifecycle
```rust
#[tokio::test]
async fn test_100_headless_terminals_lifecycle() {
    // Spawn 100 headless terminals
    // Connect to redititi
    // Run for 1 hour
    // Verify:
    // - All connect successfully
    // - All receive commands
    // - All publish output
    // - Clean shutdown works
}
```

#### Test 16.2: Headless Command Execution
```rust
#[tokio::test]
async fn test_headless_command_execution_accuracy() {
    // Execute 1000 commands across 50 headless terminals
    // Verify:
    // - Commands executed in order
    // - Exit codes captured
    // - Output streamed correctly
    // - No command loss
}
```

#### Test 16.3: Headless Output Streaming
```rust
#[tokio::test]
async fn test_headless_high_output_capture() {
    // 50 headless terminals running 'cat large_file'
    // Verify:
    // - All output captured
    // - Line-by-line publishing works
    // - Dirty detection accurate
    // - No output loss
}
```

#### Test 16.4: Headless Reconnection
```rust
#[tokio::test]
async fn test_headless_reconnection_on_server_restart() {
    // Restart redititi while headless terminals running
    // Verify:
    // - Terminals detect disconnect
    // - Automatic reconnection works
    // - Session state recoverable
    // - Minimal downtime
}
```

### Test Suite 17: Session Management Stress

**File**: `tests/integration/session_stress.rs`

#### Test 17.1: Session Isolation
```rust
#[tokio::test]
async fn test_100_isolated_sessions() {
    // 100 sessions, each with 10 panes
    // Verify:
    // - Complete isolation
    // - No cross-session leaks
    // - Independent input/output
    // - Correct channel naming
}
```

#### Test 17.2: Session Persistence
```rust
#[tokio::test]
async fn test_session_persistence_across_restarts() {
    // Create session, disconnect, reconnect
    // Verify:
    // - Session state preserved
    // - Pane registry intact
    // - Channel subscriptions restored
    // - History available
}
```

#### Test 17.3: Session Naming Stress
```rust
#[tokio::test]
async fn test_session_name_generation() {
    // Create 10k sessions rapidly
    // Verify:
    // - All unique names
    // - No collisions
    // - Memorable names (adjective-noun)
    // - Fast generation
}
```

#### Test 17.4: Nested Session Operations
```rust
#[tokio::test]
async fn test_nested_session_operations() {
    // Create/delete sessions while creating panes
    // Verify:
    // - No race conditions
    // - Clean pane deletion
    // - Registry consistency
    // - No orphaned resources
}
```

### Test Suite 18: Authentication & Security

**File**: `tests/integration/security_stress.rs`

#### Test 18.1: Concurrent Authentication
```rust
#[tokio::test]
async fn test_1000_concurrent_auth_attempts() {
    // 1000 clients authenticate simultaneously
    // Verify:
    // - All succeed with valid token
    // - All fail with invalid token
    // - No race conditions
    // - Performance acceptable
}
```

#### Test 18.2: Token Invalidation
```rust
#[tokio::test]
async fn test_token_invalidation_propagation() {
    // Invalidate token while clients connected
    // Verify:
    // - Existing clients continue working
    // - New clients rejected
    // - Graceful migration path
    // - Clear logging
}
```

#### Test 18.3: Brute Force Resistance
```rust
#[tokio::test]
async fn test_brute_force_protection() {
    // Try 10k invalid tokens
    // Verify:
    // - Rate limiting kicks in
    // - Connection closed after N attempts
    // - IP blocking (optional)
    // - No timing attacks
}
```

#### Test 18.4: Privilege Escalation
```rust
#[tokio::test]
async fn test_session_hijacking_prevention() {
    // Attempt to access other sessions
    // Verify:
    // - Access denied
    // - Session isolation enforced
    // - Audit logging happens
    // - No information leakage
}
```

### Test Suite 19: Performance & Latency

**File**: `tests/integration/performance_stress.rs`

#### Test 19.1: End-to-End Latency
```rust
#[tokio::test]
async fn test_command_injection_latency() {
    // Measure: PUBLISH → Terminal receives → Executes
    // Verify:
    // - p50 < 5ms
    // - p99 < 20ms
    // - p99.9 < 50ms
    // - No extreme outliers
}
```

#### Test 19.2: Output Capture Latency
```rust
#[tokio::test]
async fn test_output_capture_latency() {
    // Measure: Terminal output → Published → Subscriber receives
    // Verify:
    // - p50 < 10ms
    // - p99 < 30ms
    // - No lost output
    // - Ordering preserved
}
```

#### Test 19.3: Throughput Scaling
```rust
#[tokio::test]
async fn test_throughput_scaling() {
    // Measure throughput from 1 → 100 terminals
    // Verify:
    // - Linear scaling up to 50 terminals
    // - Graceful degradation beyond
    // - Clear performance metrics
    // - No sudden drops
}
```

#### Test 19.4: Resource Usage Profiling
```rust
#[tokio::test]
async fn test_resource_usage_under_load() {
    // Monitor CPU, memory, network during load
    // Verify:
    // - Memory growth linear with terminals
    // - CPU usage reasonable
    // - No memory leaks
    // - Network bandwidth efficient
}
```

### Test Suite 20: Real-World Scenarios

**File**: `tests/integration/realworld_scenarios.rs`

#### Test 20.1: CI/CD Pipeline Simulation
```rust
#[tokio::test]
async fn test_ci_cd_pipeline() {
    // 20 terminals running parallel tests
    // Orchestrated via redititi
    // Verify:
    // - All tests execute
    // - Results collected correctly
    // - Failures detected
    // - Summary generated
}
```

#### Test 20.2: Code Review Agent Swarm
```rust
#[tokio::test]
async fn test_aider_swarm() {
    // 10 Aider agents reviewing different files
    // Communicating via channels
    // Verify:
    // - All reviews complete
    // - No file conflicts
    // - Suggestions aggregated
    // - Merge conflicts detected
}
```

#### Test 20.3: Multi-Language Build System
```rust
#[tokio::test]
async fn test_polyglot_build() {
    // Build Rust, Python, Go, Node projects in parallel
    // Verify:
    // - All builds execute
    // - Dependencies resolved
    // - Build failures captured
    // - Logs aggregated
}
```

#### Test 20.4: Long-Running Data Processing
```rust
#[tokio::test]
async fn test_long_data_processing() {
    // 50 terminals processing data for 8 hours
    // Verify:
    // - All complete successfully
    // - Progress tracked
    // - No memory leaks
    // - Graceful error handling
}
```

#### Test 20.5: Interactive Multi-Agent Debugging
```rust
#[tokio::test]
async fn test_collaborative_debugging() {
    // Multiple agents debugging same issue
    // Sharing state via pub/sub
    // Verify:
    // - State synchronization works
    // - Breakpoint coordination
    // - Variable inspection shared
    // - No race conditions
}
```

---

## Chaos Engineering Tests

### Test Suite 14: Failure Injection

**File**: `tests/chaos/failure_injection.rs`

#### Test 14.1: Random Process Kill
```rust
#[test]
fn test_terminal_process_kill() {
    // Randomly kill terminal processes
    // Verify:
    // - Server detects disconnection
    // - Cleanup happens
    // - Other terminals unaffected
    // - Can recreate session
}
```

#### Test 14.2: Server Restart
```rust
#[tokio::test]
async fn test_server_restart_recovery() {
    // Restart server while clients connected
    // Verify:
    // - Clients detect disconnect
    // - Clients can reconnect
    // - State can be recovered
    // - MTTR < 5 seconds
}
```

#### Test 14.3: Network Partition
```rust
#[tokio::test]
async fn test_network_partition() {
    // Simulate network failure
    // Verify:
    // - Timeout detection works
    // - Reconnection logic works
    // - No data corruption
    // - Graceful degradation
}
```

### Test Suite 15: Resource Limits

**File**: `tests/chaos/resource_limits.rs`

#### Test 15.1: Disk Full
```rust
#[test]
fn test_disk_full_handling() {
    // Fill disk during operation
    // Verify:
    // - Clear error messages
    // - No data corruption
    // - Recovery when space available
    // - Logging stops gracefully
}
```

#### Test 15.2: Out of Memory
```rust
#[test]
fn test_oom_handling() {
    // Trigger OOM condition
    // Verify:
    // - Graceful shutdown preferred over OOM kill
    // - Error messages logged
    // - Partial state saved if possible
}
```

#### Test 15.3: CPU Starvation
```rust
#[test]
fn test_cpu_starvation() {
    // Run under heavy CPU load
    // Verify:
    // - Degraded but functional
    // - Priority handling works
    // - No deadlocks
    // - Recovery when CPU available
}
```

---

## Performance Benchmarks

### Test Suite 16: Performance Baseline

**File**: `tests/benchmarks/performance_baseline.rs`

#### Benchmark 16.1: Grid Operations
```rust
#[bench]
fn bench_grid_insert_1m_chars(b: &mut Bencher) {
    // Measure throughput of grid operations
    // Target: >10M chars/sec
}
```

#### Benchmark 16.2: ANSI Parsing
```rust
#[bench]
fn bench_ansi_parsing_throughput(b: &mut Bencher) {
    // Measure parser throughput
    // Target: >50MB/sec raw throughput
}
```

#### Benchmark 16.3: Command Processing
```rust
#[bench]
fn bench_command_processing(b: &mut Bencher) {
    // Measure redititi command throughput
    // Target: >10k commands/sec
}
```

### Test Suite 17: Latency Benchmarks

**File**: `tests/benchmarks/latency.rs`

#### Benchmark 17.1: Input Latency
```rust
#[bench]
fn bench_keystroke_to_echo_latency(b: &mut Bencher) {
    // Measure input → PTY → output latency
    // Target: <10ms p99
}
```

#### Benchmark 17.2: Command Injection Latency
```rust
#[bench]
fn bench_inject_to_execution_latency(b: &mut Bencher) {
    // Measure INJECT command latency
    // Target: <20ms p99
}
```

#### Benchmark 17.3: Screen Capture Latency
```rust
#[bench]
fn bench_capture_response_latency(b: &mut Bencher) {
    // Measure CAPTURE command latency
    // Target: <50ms p99
}
```

---

## Security Tests

### Test Suite 18: Security Hardening

**File**: `tests/security/security_tests.rs`

#### Test 18.1: Authentication Bypass Attempts
```rust
#[tokio::test]
async fn test_auth_bypass_attempts() {
    // Try common auth bypass techniques:
    // - Empty token
    // - SQL injection in token
    // - Buffer overflow attempts
    // - Timing attacks
    // Verify: All rejected correctly
}
```

#### Test 18.2: Command Injection (Security)
```rust
#[tokio::test]
async fn test_shell_injection_prevention() {
    // Try shell metacharacters in commands
    // Verify:
    // - Commands executed safely
    // - No shell escape
    // - Proper quoting
}
```

#### Test 18.3: Resource Exhaustion Attack
```rust
#[tokio::test]
async fn test_dos_resistance() {
    // Attempt DoS via:
    // - Huge messages
    // - Rapid connections
    // - Queue flooding
    // Verify: Server stays functional
}
```

---

## Real-World Scenario Tests

### Test Suite 19: Real Application Tests

**File**: `tests/scenarios/real_world.rs`

#### Test 19.1: Vim Editing Session
```rust
#[test]
fn test_vim_editing_session() {
    // Run vim, edit file, save, quit
    // Verify:
    // - All key bindings work
    // - Display renders correctly
    // - No corruption
    // - File saved correctly
}
```

#### Test 19.2: Tmux Session
```rust
#[test]
fn test_tmux_inside_titi() {
    // Run tmux inside titi
    // Verify:
    // - Nested escape sequences work
    // - Panes render correctly
    // - No conflicts
}
```

#### Test 19.3: Build Process
```rust
#[test]
fn test_cargo_build_with_output() {
    // Run cargo build
    // Verify:
    // - All output captured
    // - ANSI colors work
    // - Progress bars render
    // - Build succeeds
}
```

### Test Suite 20: Claude Code / Aider Scenarios

**File**: `tests/scenarios/ai_agents.rs`

#### Test 20.1: Claude Code Session
```rust
#[test]
fn test_claude_code_session() {
    // Simulate Claude Code agent
    // Verify:
    // - Interactive prompts work
    // - Output streamed correctly
    // - No truncation
    // - Session survives hours
}
```

#### Test 20.2: Multiple Aider Sessions
```rust
#[tokio::test]
async fn test_10_parallel_aider_sessions() {
    // Run 10 Aider instances in parallel
    // Each editing different files
    // Verify:
    // - All instances work
    // - No conflicts
    // - Git commits tracked
    // - Performance acceptable
}
```

#### Test 20.3: Agent Coordination
```rust
#[tokio::test]
async fn test_master_worker_pattern() {
    // Master agent coordinating 4 workers
    // Verify:
    // - Message passing works
    // - Task distribution correct
    // - Results aggregated properly
}
```

---

## Implementation Phases

### Week 1: Titi Terminal Hardening

**Priority 1 (Must Have):**
- ✅ Extreme output stress (Suite 1)
- ✅ Pane management stress (Suite 3)
- ✅ Resource exhaustion (Suite 5)

**Priority 2 (Should Have):**
- ✅ Rendering stress (Suite 2)
- ✅ Input stress (Suite 4)

**Priority 3 (Nice to Have):**
- ✅ 24h soak test (Suite 6.1)

### Week 2: Redititi Server Hardening

**Priority 1 (Must Have):**
- ✅ Connection stress (Suite 7)
- ✅ Command processing stress (Suite 8)
- ✅ Channel stress (Suite 9)

**Priority 2 (Should Have):**
- ✅ Registry stress (Suite 10)
- ✅ Authentication stress (Suite 11)

**Priority 3 (Nice to Have):**
- ✅ Performance benchmarks (Suite 16-17)

### Week 3: Integration Testing

**Priority 1 (Must Have):**
- ✅ Titi ↔ Redititi integration (Suite 12)
- ✅ Multi-agent coordination (Suite 13)

**Priority 2 (Should Have):**
- ✅ Real-world scenarios (Suite 19-20)

### Week 4: Chaos & Security

**Priority 1 (Must Have):**
- ✅ Failure injection (Suite 14)
- ✅ Resource limits (Suite 15)

**Priority 2 (Should Have):**
- ✅ Security hardening (Suite 18)

### Week 5: Polish & Validation

- ✅ Run all tests together
- ✅ Fix any issues found
- ✅ Update documentation
- ✅ Create test reports
- ✅ Performance tuning

---

## Success Criteria

### Titi Terminal

- ✅ **Throughput**: Handle 100k lines/sec continuous output
- ✅ **Concurrency**: Support 100 panes simultaneously
- ✅ **Stability**: Run 7+ days without memory leak (<5% growth)
- ✅ **Responsiveness**: Input latency <10ms p99
- ✅ **Rendering**: Maintain 30+ FPS under load
- ✅ **Resource Usage**:
  - Memory: <500MB for 10 panes
  - CPU: <50% for normal usage
  - GPU: <1GB VRAM

### Redititi Server

- ✅ **Scalability**: Handle 1000+ concurrent clients
- ✅ **Throughput**: Process 10k commands/sec
- ✅ **Channels**: Support 10k active channels
- ✅ **Latency**: Command response <10ms p99
- ✅ **Reliability**: 99.9% uptime
- ✅ **Resource Usage**:
  - Memory: <1GB for 1000 clients
  - CPU: <25% for normal usage
  - Network: <100MB/sec

### Integration

- ✅ **Coordination**: Support 100+ coordinated terminals
- ✅ **Reliability**: Zero message loss under normal conditions
- ✅ **Recovery**: MTTR <5 seconds after failure
- ✅ **Sustainability**: 24+ hour continuous operation
- ✅ **Security**: Pass all security tests with zero vulnerabilities

### Code Quality

- ✅ **Test Coverage**: >80% line coverage
- ✅ **Test Pass Rate**: 100% of tests pass
- ✅ **No Unsafe Code**: Zero unsafe blocks (or well-documented)
- ✅ **No Panics**: Zero panics in tests (use Result/Option)
- ✅ **Documentation**: All public APIs documented

---

## Test Execution

### Running Tests

```bash
# All tests
cargo test

# Unit tests only
cargo test --lib

# Stress tests only (may be slow)
cargo test --test stress

# Specific test suite
cargo test --test connection_stress

# Benchmarks (nightly only)
cargo +nightly bench

# Soak tests (run overnight)
cargo test --test soak_tests -- --ignored --nocapture

# With logging
RUST_LOG=debug cargo test

# With backtrace on panic
RUST_BACKTRACE=1 cargo test
```

### Continuous Integration

```yaml
# .github/workflows/battle-tests.yml
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

  soak-tests:
    runs-on: ubuntu-latest
    timeout-minutes: 1440  # 24 hours
    if: github.ref == 'refs/heads/main'
    steps:
      - uses: actions/checkout@v2
      - run: cargo test --test soak_tests -- --ignored --nocapture
```

---

## Monitoring & Metrics

### Metrics to Collect

**Titi Terminal:**
- Lines processed/sec
- Rendering FPS
- Memory usage (grid, glyph atlas, etc.)
- PTY latency
- Pane count

**Redititi Server:**
- Connections (active, total, errors)
- Commands/sec (by type)
- Channel count
- Queue depth (per channel)
- Response latency (p50, p90, p99)
- Memory usage
- CPU usage

### Alerts

- Memory usage >80%
- Response latency p99 >100ms
- Error rate >1%
- Queue depth >10k messages
- FPS <20 for >5 seconds

---

## Summary

This battle test plan will transform Titi and Redititi from functional prototypes to production-ready, mature software.

**Key Achievements:**
1. **20 Test Suites** covering every aspect
2. **100+ New Tests** for edge cases and limits
3. **Chaos Engineering** for resilience
4. **Real-World Scenarios** for validation
5. **Performance Benchmarks** for optimization
6. **Security Hardening** for production use

**Timeline**: 5 weeks intensive testing
**Outcome**: Production-ready system with 99.9% reliability

Let's break it! 💥
