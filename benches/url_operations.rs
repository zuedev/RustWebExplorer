use criterion::{black_box, criterion_group, criterion_main, Criterion};
use std::hint::black_box as hint_black_box;

// We need to copy the functions here since they're not public in the main module
// Alternatively, you could make them public in main.rs and import them

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

fn url_encode(input: &str) -> String {
    input.chars()
        .map(|c| match c {
            ' ' => "%20".to_string(),
            '"' => "%22".to_string(),
            '#' => "%23".to_string(),
            '%' => "%25".to_string(),
            '&' => "%26".to_string(),
            '+' => "%2B".to_string(),
            '?' => "%3F".to_string(),
            _ if c.is_ascii_alphanumeric() || "-_.~".contains(c) => c.to_string(),
            _ => format!("%{:02X}", c as u8),
        })
        .collect()
}

fn bench_url_decode(c: &mut Criterion) {
    let test_cases = vec![
        "hello%20world",
        "path%2Fwith%2Fslashes%2Fand%2Fmore%2Fslashes",
        "no%20encoding%20here%20just%20spaces",
        "complex%21%40%23%24%25%5E%26%2A%28%29",
        "mix%20of%20encoded%20and%20plain%20text",
        "very%20long%20path%20with%20many%20encoded%20characters%20%21%40%23%24%25%5E%26%2A%28%29%20and%20more",
    ];

    c.bench_function("url_decode_simple", |b| {
        b.iter(|| url_decode(black_box("hello%20world")))
    });

    c.bench_function("url_decode_complex", |b| {
        b.iter(|| url_decode(black_box("complex%21%40%23%24%25%5E%26%2A%28%29")))
    });

    c.bench_function("url_decode_long", |b| {
        b.iter(|| url_decode(black_box("very%20long%20path%20with%20many%20encoded%20characters%20%21%40%23%24%25%5E%26%2A%28%29%20and%20more")))
    });

    c.bench_function("url_decode_mixed", |b| {
        b.iter(|| {
            for case in &test_cases {
                hint_black_box(url_decode(black_box(case)));
            }
        })
    });
}

fn bench_url_encode(c: &mut Criterion) {
    let test_cases = vec![
        "hello world",
        "path/with/slashes/and/more/slashes",
        "no encoding here just spaces",
        "complex!@#$%^&*()",
        "mix of encoded and plain text",
        "very long path with many special characters !@#$%^&*() and more",
    ];

    c.bench_function("url_encode_simple", |b| {
        b.iter(|| url_encode(black_box("hello world")))
    });

    c.bench_function("url_encode_complex", |b| {
        b.iter(|| url_encode(black_box("complex!@#$%^&*()")))
    });

    c.bench_function("url_encode_long", |b| {
        b.iter(|| url_encode(black_box("very long path with many special characters !@#$%^&*() and more")))
    });

    c.bench_function("url_encode_mixed", |b| {
        b.iter(|| {
            for case in &test_cases {
                hint_black_box(url_encode(black_box(case)));
            }
        })
    });
}

fn bench_url_roundtrip(c: &mut Criterion) {
    let test_cases = vec![
        "hello world!@#$%^&*()",
        "file with spaces.txt",
        "path/to/file with special chars !@#$%^&*()",
    ];

    c.bench_function("url_roundtrip", |b| {
        b.iter(|| {
            for case in &test_cases {
                let encoded = url_encode(black_box(case));
                hint_black_box(url_decode(black_box(&encoded)));
            }
        })
    });
}

criterion_group!(benches, bench_url_decode, bench_url_encode, bench_url_roundtrip);
criterion_main!(benches);
