# Memory Leak Detection Test

## Overview

The comprehensive memory leak detection test is designed to challenge the system and detect memory leaks through statistical analysis of memory usage patterns over many cycles of operations.

## Test Design Philosophy

### Multi-Phase Stress Testing

The test exercises all major components that could leak memory:

1. **Pane Manager Stress**: Creates and destroys panes repeatedly
2. **Terminal Grid Stress**: Allocates and deallocates grid buffers
3. **Parser Stress**: Exercises ANSI parser state management
4. **Combined Stress**: Simulates realistic multi-pane usage

### Statistical Analysis

Rather than relying on simple before/after comparisons, the test uses statistical analysis to detect leaks:

- **Memory Growth Trend**: Compares early vs late averages
- **Monotonic Growth Detection**: Identifies sustained increasing runs
- **Delta Analysis**: Examines positive vs negative memory changes
- **Growth Percentage**: Calculates relative memory increase

## Leak Detection Criteria

A memory leak is flagged if ANY of these conditions are met:

1. **Memory Growth > 10%**
   - Late average exceeds early average by >10%
   - Indicates gradual memory accumulation

2. **Sustained Increasing Run > 20 cycles**
   - Memory increases consecutively for >20 cycles
   - Suggests resources not being freed

3. **Positive Deltas >> Negative Deltas**
   - Positive memory changes outnumber negative by 2:1
   - Indicates more allocation than deallocation

## Running the Test

### Basic Execution

```bash
# Run the comprehensive leak detection test
cargo test --release --test memory_leak_detection -- --ignored --nocapture

# Run the intentional leak test (to verify detection works)
cargo test --release --test memory_leak_detection test_intentional_leak_detection -- --ignored --nocapture
```

### Expected Output

```
╔════════════════════════════════════════════════════════════╗
║  COMPREHENSIVE MEMORY LEAK DETECTION TEST                 ║
╠════════════════════════════════════════════════════════════╣
║  This test will run 100 cycles
║  Each cycle performs 1000 operations
║  Testing: Panes, Terminals, Parsers, Grid, Atlas          ║
╚════════════════════════════════════════════════════════════╝

🔥 Warmup phase (10 cycles)...
📊 Starting memory leak detection test...

  Cycle   0: Memory: 2048 KB, Δ: +12 KB, Avg: 2048 KB
  Cycle  10: Memory: 2056 KB, Δ: +8 KB, Avg: 2052 KB
  ...

📈 Test completed in 12.345s
   Total operations: 100000

╔════════════════════════════════════════════════════════════╗
║  MEMORY LEAK ANALYSIS                                      ║
╚════════════════════════════════════════════════════════════╝

📊 Memory Statistics:
   Early average (first 25%):  2048 KB
   Late average (last 25%):    2052 KB
   Memory growth:               +4 KB (+0.20%)
   Increasing runs (>5 cycles): 2
   Max increasing run:          8 cycles
   Positive deltas:             48
   Negative deltas:             52

🔍 Leak Detection Result:
   ✅ NO SIGNIFICANT MEMORY LEAK DETECTED
   Memory usage remained stable across 100 cycles
   System appears to properly clean up resources
```

### When a Leak is Detected

```
🔍 Leak Detection Result:
   ❌ POTENTIAL MEMORY LEAK DETECTED!

   ⚠️  Memory growth 15.3% exceeds 10% threshold
   ⚠️  Sustained increasing run of 34 cycles detected
   ⚠️  Positive deltas (78) >> negative deltas (22)

   Recommended actions:
   1. Review resource cleanup in destructors
   2. Check for Arc<Mutex<>> reference cycles
   3. Verify parser state cleanup
   4. Check glyph atlas eviction policy
   5. Run with valgrind or heaptrack for details
```

## Test Architecture

### Memory Sampling

```rust
struct MemorySample {
    cycle: usize,
    memory_bytes: usize,
    memory_delta: i64,
    duration: Duration,
}
```

Each cycle records:
- Current memory usage estimate
- Delta from previous cycle
- Duration of operations

### Test Phases

#### Phase 1: Warmup (10 cycles)
- Allows JIT compilation to stabilize
- Fills caches and internal buffers
- Establishes baseline behavior
- **Not included in leak detection**

#### Phase 2: Testing (100 cycles)
- Records memory at each cycle
- Performs 1000 operations per cycle
- Forces cleanup between cycles
- Collects samples for analysis

### Operations Per Cycle

Each cycle performs 1000 operations divided into 4 phases:

**Phase 1: Pane Manager (250 ops)**
```rust
- Create pane
- Write data to pane
- Periodically destroy panes
- Cleanup remaining panes
```

**Phase 2: Terminal Grids (250 ops)**
```rust
- Create 80x24 grid
- Fill with characters
- Resize (100x30, then 60x20)
- Scroll operations
- Clear operations
```

**Phase 3: Parser (250 ops)**
```rust
- Cursor positioning sequences
- Color sequences (16-color, RGB)
- Text attributes (bold, italic)
- Screen clear sequences
- Regular text output
```

**Phase 4: Combined (250 ops)**
```rust
- 5 concurrent panes
- Interleaved operations
- Mixed ANSI sequences
- Pane switching
- Realistic usage patterns
```

## What This Test Detects

### ✅ Will Detect

1. **Grid Memory Leaks**
   - Cells not freed on grid destruction
   - Resize leaving old buffers allocated
   - Scroll history accumulation

2. **Parser State Leaks**
   - ANSI sequence state not cleaned
   - Color attribute accumulation
   - Incomplete sequence buffers

3. **Pane Manager Leaks**
   - Pane structures not freed
   - Arc reference cycles
   - Terminal instances not dropped

4. **Atlas Leaks** (when implemented)
   - Glyphs not evicted
   - Texture memory accumulation
   - Cache growing unbounded

5. **Gradual Accumulation**
   - Small leaks per operation
   - Patterns emerging over cycles
   - Memory fragmentation issues

### ❌ May Not Detect

1. **One-time Leaks**: Memory leaked once at startup
2. **Conditional Leaks**: Only leak under specific conditions not tested
3. **External Leaks**: GPU driver or system library leaks
4. **Very Slow Leaks**: Require more than 100 cycles to manifest

## Interpreting Results

### Healthy System

```
Memory growth: +2 KB (+0.1%)  ← Minimal growth
Max increasing run: 5 cycles  ← Short runs normal
Positive/Negative: 48/52     ← Balanced allocation/deallocation
```

### Suspicious System

```
Memory growth: +15 KB (+7.5%)  ← Approaching threshold
Max increasing run: 18 cycles  ← Sustained growth
Positive/Negative: 65/35       ← More allocation than deallocation
```

### Leaking System

```
Memory growth: +250 KB (+12.5%)  ← Exceeds threshold ❌
Max increasing run: 45 cycles    ← Continuous growth ❌
Positive/Negative: 85/15         ← Heavy allocation bias ❌
```

## Advanced Usage

### Customizing Test Parameters

Edit the constants in the test:

```rust
const WARMUP_CYCLES: usize = 10;      // Warmup iterations
const TEST_CYCLES: usize = 100;        // Test iterations
const OPERATIONS_PER_CYCLE: usize = 1000;  // Ops per cycle
```

For deeper testing:
```rust
const TEST_CYCLES: usize = 1000;       // 10x longer
const OPERATIONS_PER_CYCLE: usize = 5000;  // 5x more ops
```

### Integration with CI/CD

```yaml
# .github/workflows/memory-leak-test.yml
- name: Memory Leak Detection
  run: |
    cargo test --release --test memory_leak_detection -- --ignored --nocapture

# Run weekly as part of nightly tests
schedule:
  - cron: '0 0 * * 0'  # Every Sunday
```

### Using with Memory Profilers

#### Valgrind (Linux)

```bash
# Install valgrind
sudo apt-get install valgrind

# Run with leak check
valgrind --leak-check=full --show-leak-kinds=all \
    cargo test --release --test memory_leak_detection -- --ignored

# Generate suppressions for false positives
valgrind --gen-suppressions=all \
    cargo test --release --test memory_leak_detection -- --ignored
```

#### Heaptrack (Linux)

```bash
# Install heaptrack
sudo apt-get install heaptrack

# Record heap usage
heaptrack cargo test --release --test memory_leak_detection -- --ignored

# Analyze results
heaptrack_gui heaptrack.cargo.*.gz
```

#### Instruments (macOS)

```bash
# Build with debug symbols
cargo build --release --test memory_leak_detection

# Run with Instruments
instruments -t Leaks ./target/release/deps/memory_leak_detection-*
```

#### Windows Performance Analyzer

```powershell
# Use Windows Performance Toolkit
wpr -start heap -filemode

cargo test --release --test memory_leak_detection -- --ignored

wpr -stop memory_leak.etl
wpa memory_leak.etl
```

## Continuous Monitoring

### Automated Testing

Create a script `scripts/check_memory_leaks.sh`:

```bash
#!/bin/bash
set -e

echo "Running memory leak detection tests..."

# Run the test and capture output
OUTPUT=$(cargo test --release --test memory_leak_detection -- \
    --ignored --nocapture 2>&1 || true)

# Check if leak was detected
if echo "$OUTPUT" | grep -q "MEMORY LEAK DETECTED"; then
    echo "❌ MEMORY LEAK FOUND!"
    echo "$OUTPUT"
    exit 1
else
    echo "✅ No memory leaks detected"
    exit 0
fi
```

### Metrics Integration

Log results to metrics system:

```rust
// In your metrics system
METRICS.record_leak_test_result(LeakTestResult {
    cycles: 100,
    memory_growth_percent: 0.2,
    max_increasing_run: 8,
    leak_detected: false,
    timestamp: Instant::now(),
});
```

## Troubleshooting

### Test is Too Slow

- Reduce `TEST_CYCLES` to 50
- Reduce `OPERATIONS_PER_CYCLE` to 500
- Use `--release` flag for optimization

### False Positives

- Increase warmup cycles to 20
- Allow for memory allocator behavior
- Check for system background processes
- Run multiple times to confirm

### False Negatives

- Increase test cycles to 500+
- Increase operations per cycle to 5000+
- Add more specific stress for suspected leak
- Use external memory profilers

## Best Practices

1. **Run Regularly**: Include in CI/CD pipeline
2. **Use Release Mode**: `--release` for realistic performance
3. **Monitor Trends**: Track results over time
4. **Combine Tools**: Use with valgrind/heaptrack
5. **Profile Production**: Run in staging environments
6. **Document Findings**: Track leak patterns
7. **Fix Promptly**: Address leaks immediately

## Example Leak Patterns

### Pattern 1: Grid Resize Leak

```rust
// Leaky code:
impl Grid {
    pub fn resize(&mut self, cols: usize, rows: usize) {
        self.cells = vec![Cell::default(); cols * rows];
        // ❌ Old cells never explicitly dropped
        // Might be held by Arc somewhere
    }
}

// Fix:
pub fn resize(&mut self, cols: usize, rows: usize) {
    drop(std::mem::replace(&mut self.cells, Vec::new()));
    self.cells = vec![Cell::default(); cols * rows];
}
```

### Pattern 2: Parser State Leak

```rust
// Leaky code:
impl TerminalParser {
    fn handle_sequence(&mut self, seq: &[u8]) {
        let state = self.parse_state.clone();
        // ❌ State cloned but old version not cleaned
    }
}

// Fix:
fn handle_sequence(&mut self, seq: &[u8]) {
    // Reuse existing state
    self.parse_state.clear();
    self.parse_state.update(seq);
}
```

### Pattern 3: Pane Manager Cycle

```rust
// Leaky code:
struct Pane {
    terminal: Terminal,
    parent: Arc<Mutex<PaneManager>>,  // ❌ Circular reference
}

// Fix:
struct Pane {
    terminal: Terminal,
    parent: Weak<Mutex<PaneManager>>,  // ✅ Use Weak to break cycle
}
```

## Conclusion

This memory leak detection test provides comprehensive coverage of potential leak scenarios in the terminal emulator. Combined with external profiling tools and regular CI/CD execution, it ensures the system maintains stable memory usage over extended operation.

For questions or issues, see:
- `tests/stress/memory_leak_detection.rs` - Test implementation
- `src/metrics.rs` - Metrics system integration
- `TDD_TEST_PLAN.md` - Overall testing strategy
