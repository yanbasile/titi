# Testing Documentation

This document provides comprehensive information about the test suite for the Titi GPU-accelerated terminal emulator.

## Table of Contents

- [Overview](#overview)
- [Test Organization](#test-organization)
- [Running Tests](#running-tests)
- [Test Categories](#test-categories)
- [Writing New Tests](#writing-new-tests)
- [Continuous Integration](#continuous-integration)

## Overview

The test suite is designed to ensure reliability, correctness, and performance of the terminal emulator. Tests are organized into logical categories covering terminal core functionality, rendering, performance optimizations, and integration scenarios.

### Test Philosophy

1. **Comprehensive Coverage**: Test all critical paths and edge cases
2. **Fast Execution**: Unit tests should run quickly for rapid feedback
3. **Deterministic**: Tests should produce consistent results
4. **Isolated**: Tests should not depend on each other
5. **Documented**: Each test should clearly explain what it validates

## Test Organization

```
tests/
├── terminal/               # Terminal core functionality
│   ├── grid_tests.rs       # Grid operations and state management
│   ├── parser_tests.rs     # ANSI/VT escape sequence parsing
│   ├── dirty_tracking_tests.rs  # Performance optimization tracking
│   └── text_extraction_tests.rs # Copy/paste text extraction
│
├── stress/                 # Performance and stress tests
│   ├── performance.rs      # Benchmarking critical paths
│   ├── concurrency.rs      # Thread safety validation
│   └── memory_leak_detection.rs  # Memory leak detection
│
└── regression.rs           # Comprehensive regression test suite
```

## Running Tests

### Run All Tests

```bash
cargo test
```

### Run Specific Test Suites

```bash
# Terminal core tests
cargo test --test grid_tests
cargo test --test parser_tests
cargo test --test dirty_tracking_tests
cargo test --test text_extraction_tests

# Stress tests
cargo test --test performance
cargo test --test concurrency
cargo test --test memory_leak_detection

# Regression tests
cargo test --test regression
```

### Run Specific Tests

```bash
# Run a single test by name
cargo test test_put_char_marks_cell_dirty

# Run tests matching a pattern
cargo test dirty_tracking

# Run with output visible
cargo test -- --nocapture

# Run with verbose output
cargo test -- --test-threads=1 --nocapture
```

### Run Tests in Release Mode

```bash
# Useful for performance tests
cargo test --release
```

## Test Categories

### 1. Grid Tests (`grid_tests.rs`)

**Purpose**: Validate terminal grid operations including character insertion, scrolling, cursor movement, and state management.

**Key Test Areas**:
- Basic grid creation and initialization
- Character insertion and cursor advancement
- Newline and carriage return handling
- Screen clearing and line clearing
- Scrolling (up, down, regions)
- Scrollback buffer management
- Grid resizing
- Cursor saving and restoration
- Cell style management

**Example**:
```rust
#[test]
fn test_put_char() {
    let mut grid = Grid::new(80, 24);
    grid.put_char('A');
    assert_eq!(grid.cursor_pos(), (1, 0));
}
```

### 2. Parser Tests (`parser_tests.rs`)

**Purpose**: Ensure correct parsing and handling of ANSI/VT escape sequences.

**Key Test Areas**:
- Control characters (newline, tab, backspace)
- Cursor movement sequences (CUU, CUD, CUF, CUB)
- Erase sequences (ED, EL)
- SGR (Select Graphic Rendition) for colors and styles
- Private mode sequences
- Application keypad mode
- Alternative screen buffer

**Example**:
```rust
#[test]
fn test_parse_cursor_up() {
    let mut parser = TerminalParser::new(grid);
    parser.process_bytes(b"\x1b[5A"); // Move cursor up 5 lines
    assert_eq!(grid.cursor_pos().1, expected_row);
}
```

### 3. Dirty Tracking Tests (`dirty_tracking_tests.rs`)

**Purpose**: Validate the dirty rectangle tracking system for rendering optimization.

**Key Test Areas**:
- Initial dirty state on grid creation
- Cell marking on character insertion
- Line-level dirty tracking on clear operations
- Full-screen dirty on scroll operations
- Dirty state on scrollback navigation
- Clear dirty state functionality
- Dirty tracking with text wrapping

**Example**:
```rust
#[test]
fn test_put_char_marks_cell_dirty() {
    let mut grid = Grid::new(80, 24);
    grid.clear_dirty();
    grid.put_char('a');

    assert_eq!(grid.dirty_cells().len(), 1);
    assert!(grid.dirty_cells().contains(&(0, 0)));
}
```

**Why This Matters**: Dirty tracking enables the renderer to skip regenerating vertex buffers for unchanged cells, significantly improving performance during terminal updates.

### 4. Text Extraction Tests (`text_extraction_tests.rs`)

**Purpose**: Validate text extraction for copy/paste functionality.

**Key Test Areas**:
- Simple text extraction
- Multi-line text handling
- Trailing space trimming
- Empty line preservation
- Internal space preservation
- Special character handling
- Unicode character support
- Text extraction after operations (clear, resize)
- Full-width character handling

**Example**:
```rust
#[test]
fn test_extract_multiple_lines() {
    let mut grid = Grid::new(80, 3);
    // Write multiple lines...
    let text = extract_text(&grid);
    assert_eq!(text.lines().count(), 3);
}
```

**Integration**: These tests validate the `get_visible_text()` function used by the Ctrl+Shift+C copy functionality.

### 5. Renderer Tests

**Note on Renderer Testing**: The rendering subsystem (glyph atlas, text rendering, GPU state) requires full GPU context and is tested through integration tests rather than isolated unit tests. The glyph rasterization implementation is validated by:

1. **Visual Testing**: Running the terminal and verifying correct text display
2. **Integration Testing**: End-to-end rendering pipeline tests
3. **Regression Testing**: Ensuring rendering works across different scenarios

**Why This Approach**: GPU-dependent components require wgpu Device and Queue instances, making isolated unit tests impractical. Integration testing provides better coverage of actual rendering behavior.

### 6. Performance Tests (`performance.rs`)

**Purpose**: Benchmark critical performance paths.

**Key Test Areas**:
- Character insertion throughput
- Screen scrolling performance
- Parser processing speed
- Large buffer handling
- Rendering pipeline benchmarks

### 7. Concurrency Tests (`concurrency.rs`)

**Purpose**: Validate thread safety and concurrent access patterns.

**Key Test Areas**:
- Multiple pane concurrent updates
- Thread-safe terminal operations
- Lock contention scenarios
- Race condition detection

### 8. Memory Leak Detection (`memory_leak_detection.rs`)

**Purpose**: Detect memory leaks in long-running scenarios.

**Key Test Areas**:
- Scrollback buffer growth limits
- Glyph atlas memory management
- Terminal lifecycle cleanup
- Event handler cleanup

### 9. Regression Tests (`regression.rs`)

**Purpose**: Centralized test suite to prevent regressions across all subsystems.

**Sections**:
1. **Terminal Core Regression**: Basic grid operations, scrolling, clearing, resizing
2. **Dirty Tracking Regression**: Dirty state management
3. **Text Extraction Regression**: Copy/paste workflows
4. **Integration Regression**: End-to-end workflows
5. **Edge Case Regression**: Boundary conditions and special cases

**Example**:
```rust
#[test]
fn regression_copy_paste_workflow() {
    // Complete workflow: write → extract → clear → paste → verify
}
```

## Writing New Tests

### Test Structure

Follow this template for new tests:

```rust
#[test]
fn test_descriptive_name() {
    // Arrange: Set up test conditions
    let mut grid = Grid::new(80, 24);

    // Act: Perform the operation
    grid.put_char('A');

    // Assert: Verify expected outcome
    assert_eq!(grid.cursor_pos(), (1, 0));
}
```

### Naming Conventions

- **Test function names**: `test_<what>_<condition>_<expected>`
  - Example: `test_put_char_at_edge_wraps_to_next_line`

- **Regression tests**: `regression_<subsystem>_<scenario>`
  - Example: `regression_text_extraction_unicode`

### Documentation

Add doc comments to explain:
- What the test validates
- Why it's important
- Any special setup or context

```rust
/// Tests that putting a character marks the corresponding cell as dirty.
/// This is critical for rendering optimization as it allows the renderer
/// to only update changed cells.
#[test]
fn test_put_char_marks_cell_dirty() {
    // ...
}
```

### Assertions

Use descriptive assertion messages:

```rust
assert_eq!(
    actual, expected,
    "Cursor should be at column {} after writing {} chars",
    expected, char_count
);
```

## Test Coverage

### Current Coverage Areas

✅ **Terminal Core**
- Grid operations (21 tests from grid_tests.rs)
- ANSI parsing (27 tests from parser_tests.rs)
- Dirty tracking (15 tests from dirty_tracking_tests.rs)
- Text extraction (13 tests from text_extraction_tests.rs)

✅ **Rendering**
- Integration testing via visual validation
- Full pipeline testing in actual usage
- Regression testing across scenarios

✅ **Performance**
- Dirty rectangle optimization
- Scrollback efficiency
- Parser performance

✅ **Regression**
- Terminal core regression (6 tests)
- Dirty tracking regression (3 tests)
- Text extraction regression (3 tests)
- Integration workflows (2 tests)
- Edge cases (3 tests)

### Areas for Future Testing

⏳ **Planned Additions**
- Mouse event handling unit tests
- Clipboard integration tests
- Multi-pane rendering tests
- Color scheme tests
- Search in scrollback tests
- Graphics protocol tests (Sixel, Kitty)

## Continuous Integration

### Pre-commit Checks

Before committing, run:

```bash
# Run all tests
cargo test

# Check formatting
cargo fmt --check

# Run clippy
cargo clippy -- -D warnings
```

### CI Pipeline

The following tests run on every pull request:

1. **Unit Tests**: All test suites in `tests/`
2. **Integration Tests**: Full workflows
3. **Performance Tests**: Regression benchmarks
4. **Memory Tests**: Leak detection
5. **Code Coverage**: Track coverage percentage

### Performance Benchmarks

Critical paths are benchmarked to prevent performance regressions:

- Character insertion: < 1μs per character
- Screen scroll: < 100μs for full screen
- Parser processing: > 1MB/s throughput
- Glyph caching: < 10ms for full ASCII set

## Debugging Failed Tests

### View Test Output

```bash
cargo test <test_name> -- --nocapture
```

### Run Single Test

```bash
cargo test test_specific_name --test grid_tests
```

### Debug Mode

```bash
# Build and run in debug mode for better error messages
cargo test --test grid_tests
```

### Common Issues

1. **Timing Issues**: Some tests may be timing-sensitive
   - Solution: Use `--test-threads=1` to run sequentially

2. **Font Issues**: Glyph tests may fail if system fonts are missing
   - Solution: Tests should handle missing glyphs gracefully

3. **Platform Differences**: Some tests may behave differently on Windows/Linux/Mac
   - Solution: Use conditional compilation `#[cfg(target_os = "...")]`

## Test Metrics

### Success Criteria

- ✅ All tests pass on CI
- ✅ No memory leaks detected
- ✅ Performance benchmarks within acceptable range
- ✅ Code coverage > 80%

### Current Status

```
Terminal Core:    21/21 tests passing ✓
Parser:           27/27 tests passing ✓
Dirty Tracking:   15/15 tests passing ✓
Text Extraction:  13/13 tests passing ✓
Regression:       17/17 tests passing ✓
-------------------------
Total:            93/93 tests passing ✓
```

## Contributing

When adding new features:

1. **Write tests first** (TDD approach recommended)
2. **Add regression tests** for bug fixes
3. **Update this documentation** with new test categories
4. **Ensure all tests pass** before submitting PR

## References

- [Rust Testing Best Practices](https://doc.rust-lang.org/book/ch11-00-testing.html)
- [cargo test documentation](https://doc.rust-lang.org/cargo/commands/cargo-test.html)
- Terminal emulator testing: VT100/ANSI test suites

---

**Last Updated**: 2025-11-25
**Test Suite Version**: 1.0
**Maintainer**: Titi Development Team
