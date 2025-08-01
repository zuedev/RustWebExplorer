use std::fs;
use std::io::{Read, Write};
use std::net::TcpListener;
use std::path::{Path, PathBuf};

fn parse_requested_path(request: &str) -> Option<PathBuf> {
    let mut lines = request.lines();
    if let Some(first_line) = lines.next() {
        if let Some(path) = first_line.split_whitespace().nth(1) {
            let tail = path.trim_start_matches('/');
            let root = match std::env::current_dir() {
                Ok(dir) => dir,
                Err(_) => return None,
            };
            if !tail.is_empty() {
                let requested_path = Path::new(tail);
                if requested_path.is_absolute() {
                    return None;
                }
                if requested_path.components().any(|c| matches!(c, std::path::Component::ParentDir)) {
                    return None;
                }
                let current_path = root.join(requested_path);
                match current_path.canonicalize() {
                    Ok(resolved) => {
                        let normalized_resolved = resolved.strip_prefix(r"\\?\").unwrap_or(&resolved);
                        let normalized_root = match root.canonicalize() {
                            Ok(canonical_root) => {
                                let canonical_root_owned = canonical_root.strip_prefix(r"\\?\").unwrap_or(&canonical_root).to_path_buf();
                                canonical_root_owned
                            },
                            Err(_) => return None,
                        };
                        if normalized_resolved.starts_with(&normalized_root) {
                            return Some(resolved);
                        } else {
                            return None;
                        }
                    }
                    Err(_) => return None,
                }
            }
            return Some(root);
        }
    }
    None
}

fn generate_file_response(file_path: &Path) -> String {
    match fs::read_to_string(file_path) {
        Ok(contents) => format!(
            "HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\n\r\n{}",
            contents
        ),
        Err(_) => "HTTP/1.1 500 Internal Server Error\r\n\r\nError reading file".to_string(),
    }
}

fn generate_directory_response(dir_path: &Path, tail: &str) -> String {
    let paths = match fs::read_dir(dir_path) {
        Ok(entries) => entries.filter_map(Result::ok).map(|entry| entry.path()).collect::<Vec<_>>(),
        Err(_) => {
            return "HTTP/1.1 500 Internal Server Error\r\n\r\nError reading directory".to_string();
        }
    };

    let mut directories = vec![];
    let mut files = vec![];

    for path in paths {
        if fs::metadata(&path).map(|m| m.is_dir()).unwrap_or(false) {
            directories.push(path.clone());
        } else {
            files.push(path.clone());
        }
    }

    let mut response = format!(
        "<!DOCTYPE html>
        <html>
        <head>
            <style>
                body {{ font-family: monospace; }}
            </style>
        </head>
        <body>
        <h1>Contents of: {}</h1>",
        if dir_path.display().to_string().contains(r"\\?\") {
            "<abbr title=\"'\\\\?\\' is a Windows MAX_PATH feature that allows paths longer than 260 characters\">".to_string() + &dir_path.display().to_string() + "</abbr>"
        } else {
            dir_path.display().to_string()
        }
    );

    if let Some(_) = dir_path.parent() {
        if dir_path != std::env::current_dir().expect("Failed to get current working directory") {
            let parent_url = tail.rsplit_once('/').map(|(base, _)| base).unwrap_or("");
            response.push_str(&format!(
                "&#8592; <a href=\"/{}\">Parent Directory</a><br><br>",
                parent_url
            ));
        }
    }

    for dir in directories {
        if let Some(name) = dir.file_name().and_then(|n| n.to_str()) {
            let rel_path = Path::new(tail).join(name);
            response.push_str(&format!(
                "&#128193; <a href=\"/{}\">{}</a><br>",
                rel_path.display(),
                name
            ));
        }
    }

    for file in files {
        if let Some(name) = file.file_name().and_then(|n| n.to_str()) {
            let rel_path = Path::new(tail).join(name);
            response.push_str(&format!(
                "&#128196; <a href=\"/{}\">{}</a><br>",
                rel_path.display(),
                name
            ));
        }
    }

    response.push_str("</body></html>");

    format!("HTTP/1.1 200 OK\r\nContent-Type: text/html\r\n\r\n{}", response)
}

fn handle_request(request: &str) -> String {
    if let Some(current_path) = parse_requested_path(request) {
        if current_path.is_file() {
            return generate_file_response(&current_path);
        } else if current_path.is_dir() {
            let tail = request
                .lines()
                .next()
                .and_then(|line| line.split_whitespace().nth(1))
                .unwrap_or("")
                .trim_start_matches('/');
            return generate_directory_response(&current_path, tail);
        }
    }
    "HTTP/1.1 400 Bad Request\r\n\r\nBad Request".to_string()
}

fn main() -> std::io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:8080")?;

    for stream in listener.incoming() {
        match stream {
            Ok(mut stream) => {
                std::thread::spawn(move || {
                    let mut buffer = [0; 4096];
                    let bytes_read = match stream.read(&mut buffer) {
                        Ok(n) => n,
                        Err(_) => return,
                    };
                    let request = String::from_utf8_lossy(&buffer[..bytes_read]);
                    let response = handle_request(&request);
                    if let Err(_) = stream.write_all(response.as_bytes()) {}
                    if let Err(_) = stream.flush() {}
                });
            }
            Err(_) => {}
        }
    }

    Ok(())
}