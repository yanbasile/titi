#!/bin/bash
# Performance profiling script for Titi terminal emulator
# Generates flamegraphs for hot path analysis

set -e

echo "╔════════════════════════════════════════════════════════════╗"
echo "║  Titi Performance Profiling                               ║"
echo "╚════════════════════════════════════════════════════════════╝"
echo ""

# Create profiling output directory
mkdir -p profile_results

echo "📊 Profiling stress tests..."
echo ""

# Profile performance test
echo "1. Profiling performance stress test..."
cargo flamegraph --test performance --output profile_results/performance_flamegraph.svg -- --nocapture 2>&1 | grep -v "^warning:" | tail -20

echo ""
echo "2. Profiling concurrency test..."
cargo flamegraph --test concurrency --output profile_results/concurrency_flamegraph.svg -- --nocapture 2>&1 | grep -v "^warning:" | tail -20

echo ""
echo "3. Profiling memory leak detection..."
cargo flamegraph --test memory_leak_detection --output profile_results/memory_flamegraph.svg -- --nocapture 2>&1 | grep -v "^warning:" | tail -20

echo ""
echo "╔════════════════════════════════════════════════════════════╗"
echo "║  Profiling Complete!                                       ║"
echo "╚════════════════════════════════════════════════════════════╝"
echo ""
echo "📈 Flamegraph SVG files generated in profile_results/:"
echo "   - performance_flamegraph.svg"
echo "   - concurrency_flamegraph.svg"
echo "   - memory_flamegraph.svg"
echo ""
echo "Open these files in a browser to analyze hot paths."
