# Observation Tools Client Benchmarks

This document describes the performance benchmarks for the Observation Tools client library.

## Overview

The benchmarks measure upload performance for observations with different payload sizes and quantities. They help track performance regressions and understand the impact of the blob storage threshold (64KB) on upload times.

## Running Benchmarks

To run all benchmarks:

```bash
cargo bench --bench upload_benchmarks
```

To run a specific benchmark group:

```bash
# Single small observation benchmarks
cargo bench --bench upload_benchmarks -- single_small_observation

# Single large observation benchmarks
cargo bench --bench upload_benchmarks -- single_large_observation

# Throughput benchmarks for small observations
cargo bench --bench upload_benchmarks -- throughput_small_observations

# Throughput benchmarks for large observations
cargo bench --bench upload_benchmarks -- throughput_large_observations

# Threshold boundary benchmarks
cargo bench --bench upload_benchmarks -- threshold_boundary
```

## Benchmark Groups

### 1. Single Small Observation (`bench_single_small_observation`)

**Purpose:** Measures the time to upload a single small observation (1KB payload).

**Metrics:**
- Upload time per observation
- Throughput (bytes/second)

**Key Insights:**
- Small payloads are sent inline with observation metadata
- No separate blob upload required
- Lower latency, suitable for frequent small observations

### 2. Single Large Observation (`bench_single_large_observation`)

**Purpose:** Measures the time to upload a single large observation with varying payload sizes.

**Test Sizes:**
- 100KB
- 500KB
- 1MB
- 5MB

**Metrics:**
- Upload time per observation
- Throughput (bytes/second) for each size

**Key Insights:**
- Large payloads (≥64KB) trigger blob storage
- Two-tier upload: metadata first, then blob
- Higher latency due to separate blob upload
- Tests realistic large payload scenarios

### 3. Throughput - Small Observations (`bench_throughput_small_observations`)

**Purpose:** Measures maximum throughput when uploading multiple small observations (1KB each).

**Batch Sizes:** 10, 50, 100, 500, 1000 observations

**Metrics:**
- Total upload time for batch
- Throughput (bytes/second)
- Observations per second

**Key Insights:**
- Tests batching efficiency
- Background uploader batches up to 100 observations
- Measures system performance under high observation volume

### 4. Throughput - Large Observations (`bench_throughput_large_observations`)

**Purpose:** Measures maximum throughput when uploading multiple large observations (100KB each).

**Batch Sizes:** 5, 10, 25, 50 observations

**Metrics:**
- Total upload time for batch
- Throughput (bytes/second)
- Observations per second

**Key Insights:**
- Tests blob storage performance at scale
- Each observation requires separate blob upload
- Identifies bottlenecks in concurrent blob uploads

### 5. Threshold Boundary (`bench_threshold_boundary`)

**Purpose:** Compares performance at the blob storage threshold boundary (64KB).

**Test Cases:**
- 63KB (below threshold, inline storage)
- 64KB (at threshold, blob storage)
- 65KB (above threshold, blob storage)

**Metrics:**
- Upload time for each case
- Throughput comparison

**Key Insights:**
- Quantifies performance impact of blob storage
- Helps validate the 64KB threshold choice
- Shows latency difference between inline and blob storage

## Understanding Results

### Output Format

Criterion produces detailed reports including:
- Mean execution time with confidence intervals
- Throughput measurements (bytes/second, observations/second)
- Comparison with previous runs (if available)
- HTML reports with graphs (in `target/criterion/`)

### Interpreting Metrics

**Time per Observation:**
- Lower is better
- Includes network latency, serialization, and server processing

**Throughput (bytes/second):**
- Higher is better
- Indicates data transfer efficiency
- Compare across payload sizes to understand scaling

**Threshold Impact:**
- Compare inline vs. blob storage benchmarks
- Expect higher latency for blob storage (separate HTTP request)
- Blob storage should still be acceptable for large payloads

## Performance Optimization Tips

Based on benchmark results, consider:

1. **Batching:** Use natural batching (background uploader handles this automatically)
2. **Payload Size:** Keep payloads under 64KB when possible to avoid blob storage overhead
3. **Concurrency:** Multiple executions can run concurrently without interference
4. **Network:** Performance depends on network latency to the server

## Continuous Integration

To track performance over time:

1. **Baseline:** Run benchmarks and save results
   ```bash
   cargo bench --bench upload_benchmarks -- --save-baseline main
   ```

2. **Compare:** After changes, compare against baseline
   ```bash
   cargo bench --bench upload_benchmarks -- --baseline main
   ```

3. **Bencher.dev Integration:** (Optional) Push results to bencher.dev for tracking
   ```bash
   # Install bencher CLI
   cargo install bencher_cli

   # Run benchmarks and upload results
   bencher run "cargo bench --bench upload_benchmarks"
   ```

## Troubleshooting

**Benchmarks fail to compile:**
- Ensure all dependencies are up to date: `cargo update`
- Clean build artifacts: `cargo clean`

**Benchmarks are slow:**
- Default measurement time is 10-20 seconds per benchmark
- Reduce sample size for faster (but less accurate) results
- Run specific benchmark groups instead of all

**High variance in results:**
- Close other applications to reduce system load
- Run benchmarks multiple times and compare
- Check network stability (benchmarks use local server, but network stack is still involved)

## Benchmark Infrastructure

**Test Server:**
- Spins up a local observation-tools-server instance
- Uses temporary directory for storage (cleaned up automatically)
- Binds to random available port to avoid conflicts

**Runtime:**
- Uses Tokio async runtime
- Each benchmark gets fresh server instance
- Client shutdown ensures clean state between runs

## Contributing

When modifying the client library:
1. Run benchmarks before and after changes
2. Compare results to ensure no regressions
3. Document significant performance changes
4. Update benchmarks if API changes affect them
