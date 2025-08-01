use criterion::{black_box, criterion_group, criterion_main, Criterion};
use std::path::Path;
use std::hint::black_box as hint_black_box;

fn get_mime_type(file_path: &Path) -> &'static str {
    if let Some(extension) = file_path.extension().and_then(|ext| ext.to_str()) {
        match extension.to_lowercase().as_str() {
            // Image types
            "jpg" | "jpeg" => "image/jpeg",
            "png" => "image/png",
            "gif" => "image/gif",
            "bmp" => "image/bmp",
            "webp" => "image/webp",
            "svg" => "image/svg+xml",
            "ico" => "image/x-icon",
            "tiff" | "tif" => "image/tiff",
            // Video types
            "mp4" => "video/mp4",
            "webm" => "video/webm",
            "ogg" => "video/ogg",
            "mov" => "video/quicktime",
            "avi" => "video/x-msvideo",
            "mkv" => "video/x-matroska",
            "wmv" => "video/x-ms-wmv",
            "flv" => "video/x-flv",
            "m4v" => "video/x-m4v",
            _ => "text/plain",
        }
    } else {
        "text/plain"
    }
}

fn is_image_file(file_path: &Path) -> bool {
    if let Some(extension) = file_path.extension().and_then(|ext| ext.to_str()) {
        matches!(
            extension.to_lowercase().as_str(),
            "jpg" | "jpeg" | "png" | "gif" | "bmp" | "webp" | "svg" | "ico" | "tiff" | "tif"
        )
    } else {
        false
    }
}

fn is_video_file(file_path: &Path) -> bool {
    if let Some(extension) = file_path.extension().and_then(|ext| ext.to_str()) {
        matches!(
            extension.to_lowercase().as_str(),
            "mp4" | "webm" | "ogg" | "mov" | "avi" | "mkv" | "wmv" | "flv" | "m4v"
        )
    } else {
        false
    }
}

fn bench_mime_type_detection(c: &mut Criterion) {
    let test_files = vec![
        Path::new("test.jpg"),
        Path::new("test.png"),
        Path::new("test.gif"),
        Path::new("test.mp4"),
        Path::new("test.webm"),
        Path::new("test.txt"),
        Path::new("file_without_extension"),
        Path::new("test.JPEG"), // uppercase
        Path::new("test.MP4"),  // uppercase
    ];

    c.bench_function("mime_type_jpg", |b| {
        b.iter(|| get_mime_type(black_box(Path::new("test.jpg"))))
    });

    c.bench_function("mime_type_png", |b| {
        b.iter(|| get_mime_type(black_box(Path::new("test.png"))))
    });

    c.bench_function("mime_type_mp4", |b| {
        b.iter(|| get_mime_type(black_box(Path::new("test.mp4"))))
    });

    c.bench_function("mime_type_unknown", |b| {
        b.iter(|| get_mime_type(black_box(Path::new("test.unknown"))))
    });

    c.bench_function("mime_type_no_extension", |b| {
        b.iter(|| get_mime_type(black_box(Path::new("file_without_extension"))))
    });

    c.bench_function("mime_type_mixed", |b| {
        b.iter(|| {
            for file in &test_files {
                hint_black_box(get_mime_type(black_box(file)));
            }
        })
    });
}

fn bench_file_type_detection(c: &mut Criterion) {
    let test_files = vec![
        Path::new("test.jpg"),
        Path::new("test.png"),
        Path::new("test.gif"),
        Path::new("test.mp4"),
        Path::new("test.webm"),
        Path::new("test.avi"),
        Path::new("test.txt"),
        Path::new("file_without_extension"),
    ];

    c.bench_function("is_image_file", |b| {
        b.iter(|| {
            for file in &test_files {
                hint_black_box(is_image_file(black_box(file)));
            }
        })
    });

    c.bench_function("is_video_file", |b| {
        b.iter(|| {
            for file in &test_files {
                hint_black_box(is_video_file(black_box(file)));
            }
        })
    });

    c.bench_function("combined_file_checks", |b| {
        b.iter(|| {
            for file in &test_files {
                let path = black_box(file);
                hint_black_box(get_mime_type(path));
                hint_black_box(is_image_file(path));
                hint_black_box(is_video_file(path));
            }
        })
    });
}

fn bench_case_insensitive_matching(c: &mut Criterion) {
    let mixed_case_files = vec![
        Path::new("test.JPG"),
        Path::new("test.PNG"),
        Path::new("test.GIF"),
        Path::new("test.MP4"),
        Path::new("test.WEBM"),
        Path::new("test.AVI"),
        Path::new("Test.JpG"),
        Path::new("TEST.mp4"),
    ];

    c.bench_function("case_insensitive_mime", |b| {
        b.iter(|| {
            for file in &mixed_case_files {
                hint_black_box(get_mime_type(black_box(file)));
            }
        })
    });

    c.bench_function("case_insensitive_image", |b| {
        b.iter(|| {
            for file in &mixed_case_files {
                hint_black_box(is_image_file(black_box(file)));
            }
        })
    });

    c.bench_function("case_insensitive_video", |b| {
        b.iter(|| {
            for file in &mixed_case_files {
                hint_black_box(is_video_file(black_box(file)));
            }
        })
    });
}

criterion_group!(benches, bench_mime_type_detection, bench_file_type_detection, bench_case_insensitive_matching);
criterion_main!(benches);
