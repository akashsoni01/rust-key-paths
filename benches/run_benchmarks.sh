#!/bin/bash

# Benchmark Performance Report Generator
# Compares KeyPaths vs Direct Unwrap Performance

set -e

echo "🔬 KeyPaths Performance Benchmark Report"
echo "=========================================="
echo ""
echo "Running comprehensive benchmarks comparing KeyPaths vs Direct Unwrap..."
echo ""

# Run benchmarks
cargo bench --bench keypath_vs_unwrap

echo ""
echo "✅ Benchmarks completed!"
echo ""
echo "📊 Results are available in:"
echo "   - target/criterion/keypath_vs_unwrap/"
echo "   - HTML reports: target/criterion/keypath_vs_unwrap/*/report/index.html"
echo ""
echo "To view results, open the HTML files in your browser."

