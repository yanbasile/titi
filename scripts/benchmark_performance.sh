#!/bin/bash
# Performance benchmarking script for Titi terminal emulator
# Runs tests and collects timing metrics

set -e

echo "╔════════════════════════════════════════════════════════════╗"
echo "║  Titi Performance Benchmarking                            ║"
echo "╚════════════════════════════════════════════════════════════╝"
echo ""

# Create benchmark output directory
mkdir -p benchmark_results
TIMESTAMP=$(date +%Y%m%d_%H%M%S)
REPORT="benchmark_results/benchmark_${TIMESTAMP}.txt"

echo "Performance Benchmark Report" > "$REPORT"
echo "Generated: $(date)" >> "$REPORT"
echo "=====================================" >> "$REPORT"
echo "" >> "$REPORT"

echo "📊 Running performance benchmarks..."
echo ""

# Benchmark 1: Grid operations
echo "1. Benchmarking grid operations..."
echo "## Grid Operations" >> "$REPORT"
/usr/bin/time -v cargo test --test grid_tests -- --nocapture 2>&1 | tee -a "$REPORT" | grep -E "(test result|elapsed)"
echo "" >> "$REPORT"

# Benchmark 2: Parser performance
echo "2. Benchmarking parser..."
echo "## Parser Performance" >> "$REPORT"
/usr/bin/time -v cargo test --test parser_tests -- --nocapture 2>&1 | tee -a "$REPORT" | grep -E "(test result|elapsed)"
echo "" >> "$REPORT"

# Benchmark 3: Stress tests
echo "3. Benchmarking stress tests..."
echo "## Stress Tests" >> "$REPORT"
/usr/bin/time -v cargo test --test performance -- --nocapture 2>&1 | tee -a "$REPORT" | grep -E "(test result|elapsed)"
echo "" >> "$REPORT"

# Benchmark 4: Resource usage
echo "4. Benchmarking resource exhaustion..."
echo "## Resource Exhaustion" >> "$REPORT"
/usr/bin/time -v cargo test --test battle_resources -- --ignored --nocapture 2>&1 | tee -a "$REPORT" | grep -E "(test result|elapsed|Created|commands/sec)"
echo "" >> "$REPORT"

echo ""
echo "╔════════════════════════════════════════════════════════════╗"
echo "║  Benchmarking Complete!                                    ║"
echo "╚════════════════════════════════════════════════════════════╝"
echo ""
echo "📈 Results saved to: $REPORT"
echo ""
echo "Key Metrics to Review:"
echo "- Test execution times"
echo "- Memory usage (Maximum resident set size)"
echo "- Commands per second throughput"
