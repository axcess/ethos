#!/usr/bin/env bash
# Phase 1 benchmark for the Mac mini (M4, 16 GB). Run from the repo root:
#   bash tools/bench/run_mac_bench.sh
# Writes docs/reports/bench-<host>-<date>.md. Please send the file back (or commit it).
set -euo pipefail
cd "$(dirname "$0")/../.."
command -v cargo >/dev/null || { echo "Install Rust first: https://rustup.rs"; exit 1; }
cargo build --release -q
out="docs/reports/bench-$(hostname -s)-$(date +%Y%m%d).md"
{
  echo "# Benchmark: $(hostname -s), $(date)"
  echo
  echo '```'
  sw_vers 2>/dev/null || uname -a
  sysctl -n machdep.cpu.brand_string 2>/dev/null || true
  echo "cores: $(sysctl -n hw.ncpu 2>/dev/null || nproc)"
  echo "memory: $(( $(sysctl -n hw.memsize 2>/dev/null || echo 0) / 1073741824 )) GB"
  echo '```'
  echo
  echo "## Throughput by population (single core)"
  echo
  ./target/release/tank bench --pops 100,200,400,800,1600 --cycles 2
  echo
  echo "## Determinism (must print MATCH twice; hashes should equal the Linux reference in docs/reports/phase-1-report.md)"
  echo
  echo '```'
  ./target/release/tank determinism --seed 3 --cycles 8
  echo '```'
  echo
  echo "## Batch throughput (all cores)"
  echo
  echo '```'
  /usr/bin/time -l ./target/release/tank batch --seeds 1..8 --cycles 40 --out runs/mac-bench >/dev/null 2>runs/mac-bench.time || true
  tail -20 runs/mac-bench.time
  echo '```'
} | tee "$out"
echo "wrote $out"
