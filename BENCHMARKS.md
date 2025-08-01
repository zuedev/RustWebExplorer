# Rust Web Explorer Benchmarks

This project includes comprehensive benchmarks to measure the performance of critical functions in the web server.

## Setup

The benchmarks use [Criterion.rs](https://bheisler.github.io/criterion.rs/book/index.html), a statistics-driven benchmarking library for Rust.

## Running Benchmarks

### All Benchmarks

```bash
cargo bench
```

### Individual Benchmark Suites

#### URL Operations Benchmarks

Tests the performance of URL encoding and decoding functions:

```bash
cargo bench --bench url_operations
```

#### Path Parsing Benchmarks

Tests the performance of HTTP request path parsing and security validation:

```bash
cargo bench --bench path_parsing
```

#### File Operations Benchmarks

Tests the performance of MIME type detection and file type classification:

```bash
cargo bench --bench file_operations
```

## VS Code Tasks

This project includes VS Code tasks for easy benchmark execution:

- **Run Benchmarks**: Executes all benchmarks
- **Run URL Operations Benchmarks**: Executes only URL operation benchmarks
- **Run Path Parsing Benchmarks**: Executes only path parsing benchmarks
- **Run File Operations Benchmarks**: Executes only file operation benchmarks

Access these via `Ctrl+Shift+P` → "Tasks: Run Task"

## Benchmark Output

Criterion generates detailed reports including:

- Mean execution time with confidence intervals
- Performance comparison with previous runs
- Statistical analysis of outliers
- HTML reports (when available) in `target/criterion/`

## Performance Targets

Key functions to monitor:

- `url_decode`: Should handle typical web requests (< 1µs for simple paths)
- `url_encode`: Should efficiently encode special characters
- `parse_requested_path`: Critical for request processing performance
- `get_mime_type`: Should be fast for file serving

## Optimization Guidelines

1. **URL Operations**: Focus on minimizing string allocations
2. **Path Parsing**: Optimize path validation and canonicalization
3. **File Operations**: Ensure efficient extension matching

## Adding New Benchmarks

To add new benchmarks:

1. Add benchmark functions to the appropriate file in `benches/`
2. Include them in the `criterion_group!` macro
3. Update this README with the new benchmark description

## Dependencies

- `criterion = "0.5"`: Main benchmarking framework
