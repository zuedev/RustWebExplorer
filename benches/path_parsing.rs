use criterion::{black_box, criterion_group, criterion_main, Criterion};
use std::path::{Path, PathBuf};
use std::hint::black_box as hint_black_box;

// Copy the url_decode function for use in path parsing
fn url_decode(input: &str) -> String {
    let mut result = String::new();
    let mut chars = input.chars();
    
    while let Some(ch) = chars.next() {
        if ch == '%' {
            let hex1 = chars.next().unwrap_or('0');
            let hex2 = chars.next().unwrap_or('0');
            if let Ok(byte) = u8::from_str_radix(&format!("{}{}", hex1, hex2), 16) {
                result.push(byte as char);
            } else {
                result.push(ch);
                result.push(hex1);
                result.push(hex2);
            }
        } else {
            result.push(ch);
        }
    }
    result
}

// Simplified version of parse_requested_path for benchmarking
fn parse_requested_path_bench(request: &str) -> Option<PathBuf> {
    let mut lines = request.lines();
    if let Some(first_line) = lines.next() {
        // Parse HTTP request line: "GET /path HTTP/1.1"
        let parts: Vec<&str> = first_line.splitn(3, ' ').collect();
        if parts.len() >= 2 {
            let url_path = parts[1];
            let decoded_path = url_decode(url_path);
            let tail = decoded_path.trim_start_matches('/');
            if let Ok(root) = std::env::current_dir() {
                if !tail.is_empty() {
                    let requested_path = Path::new(tail);
                    if requested_path.is_absolute() {
                        return None;
                    }
                    if requested_path.components().any(|c| matches!(c, std::path::Component::ParentDir)) {
                        return None;
                    }
                    let current_path = root.join(requested_path);
                    if let Ok(resolved) = current_path.canonicalize() {
                        let normalized_resolved = resolved.strip_prefix(r"\\?\").unwrap_or(&resolved);
                        if let Ok(canonical_root) = root.canonicalize() {
                            let normalized_root = canonical_root.strip_prefix(r"\\?\").unwrap_or(&canonical_root);
                            if normalized_resolved.starts_with(normalized_root) {
                                return Some(resolved);
                            }
                        }
                    }
                }
                return Some(root);
            }
        }
    }
    None
}

fn bench_parse_requested_path(c: &mut Criterion) {
    let test_requests = vec![
        "GET / HTTP/1.1\r\nHost: localhost\r\n\r\n",
        "GET /src/main.rs HTTP/1.1\r\nHost: localhost\r\n\r\n",
        "GET /tests%2Fsample.jpg HTTP/1.1\r\nHost: localhost\r\n\r\n",
        "GET /directory%20with%20spaces HTTP/1.1\r\nHost: localhost\r\n\r\n",
        "GET /very/deep/nested/path/to/some/file.txt HTTP/1.1\r\nHost: localhost\r\n\r\n",
    ];

    c.bench_function("parse_root_path", |b| {
        b.iter(|| parse_requested_path_bench(black_box("GET / HTTP/1.1\r\nHost: localhost\r\n\r\n")))
    });

    c.bench_function("parse_simple_path", |b| {
        b.iter(|| parse_requested_path_bench(black_box("GET /src/main.rs HTTP/1.1\r\nHost: localhost\r\n\r\n")))
    });

    c.bench_function("parse_encoded_path", |b| {
        b.iter(|| parse_requested_path_bench(black_box("GET /tests%2Fsample.jpg HTTP/1.1\r\nHost: localhost\r\n\r\n")))
    });

    c.bench_function("parse_spaces_path", |b| {
        b.iter(|| parse_requested_path_bench(black_box("GET /directory%20with%20spaces HTTP/1.1\r\nHost: localhost\r\n\r\n")))
    });

    c.bench_function("parse_deep_path", |b| {
        b.iter(|| parse_requested_path_bench(black_box("GET /very/deep/nested/path/to/some/file.txt HTTP/1.1\r\nHost: localhost\r\n\r\n")))
    });

    c.bench_function("parse_mixed_requests", |b| {
        b.iter(|| {
            for request in &test_requests {
                hint_black_box(parse_requested_path_bench(black_box(request)));
            }
        })
    });
}

fn bench_path_security_checks(c: &mut Criterion) {
    let malicious_requests = vec![
        "GET /../etc/passwd HTTP/1.1\r\nHost: localhost\r\n\r\n",
        "GET /../../secret HTTP/1.1\r\nHost: localhost\r\n\r\n",
        "GET /C:/Windows/System32 HTTP/1.1\r\nHost: localhost\r\n\r\n",
        "GET /..%2F..%2Fetc%2Fpasswd HTTP/1.1\r\nHost: localhost\r\n\r\n",
    ];

    c.bench_function("security_checks", |b| {
        b.iter(|| {
            for request in &malicious_requests {
                hint_black_box(parse_requested_path_bench(black_box(request)));
            }
        })
    });
}

criterion_group!(benches, bench_parse_requested_path, bench_path_security_checks);
criterion_main!(benches);
