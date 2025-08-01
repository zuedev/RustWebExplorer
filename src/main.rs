use std::fs;
use std::io::{Read, Write};
use std::net::TcpListener;
use std::path::{Path, PathBuf};

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

fn parse_requested_path(request: &str) -> Option<PathBuf> {
    let mut lines = request.lines();
    if let Some(first_line) = lines.next() {
        // Parse HTTP request line: "GET /path HTTP/1.1"
        let parts: Vec<&str> = first_line.splitn(3, ' ').collect();
        if parts.len() >= 2 {
            let url_path = parts[1];
            let decoded_path = url_decode(url_path);
            let tail = decoded_path.trim_start_matches('/');
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

fn generate_file_response(file_path: &Path) -> Vec<u8> {
    let mime_type = get_mime_type(file_path);
    
    if is_image_file(file_path) || is_video_file(file_path) {
        // Handle image and video files as binary
        match fs::read(file_path) {
            Ok(contents) => {
                let header = format!(
                    "HTTP/1.1 200 OK\r\nContent-Type: {}\r\nContent-Length: {}\r\n\r\n",
                    mime_type,
                    contents.len()
                );
                let mut response = header.into_bytes();
                response.extend_from_slice(&contents);
                response
            }
            Err(_) => format!("HTTP/1.1 500 Internal Server Error\r\n\r\nError reading {} file", if is_image_file(file_path) { "image" } else { "video" }).as_bytes().to_vec(),
        }
    } else {
        // Handle text files
        match fs::read_to_string(file_path) {
            Ok(contents) => format!(
                "HTTP/1.1 200 OK\r\nContent-Type: {}\r\n\r\n{}",
                mime_type, contents
            ).into_bytes(),
            Err(_) => "HTTP/1.1 500 Internal Server Error\r\n\r\nError reading file".as_bytes().to_vec(),
        }
    }
}

fn generate_directory_response(dir_path: &Path, tail: &str) -> Vec<u8> {
    let paths = match fs::read_dir(dir_path) {
        Ok(entries) => entries.filter_map(Result::ok).map(|entry| entry.path()).collect::<Vec<_>>(),
        Err(_) => {
            return "HTTP/1.1 500 Internal Server Error\r\n\r\nError reading directory".as_bytes().to_vec();
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
                body {{ font-family: monospace; background: #fff; color: #111; }}
                a {{ color: #0000ff; }}
                @media (prefers-color-scheme: dark) {{
                    body {{ background: #111111; color: #ffffff; }}
                    a {{ color: #00ff00; }}
                }}
            </style>
        </head>
        <body>
        <h1>{}</h1>",
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
                url_encode(parent_url)
            ));
        }
    }

    for dir in directories {
        if let Some(name) = dir.file_name().and_then(|n| n.to_str()) {
            let rel_path = Path::new(tail).join(name);
            let encoded_path = url_encode(&rel_path.display().to_string());
            response.push_str(&format!(
                "&#128193; <a href=\"/{}\">{}</a><br>",
                encoded_path,
                name
            ));
        }
    }

    for file in files {
        if let Some(name) = file.file_name().and_then(|n| n.to_str()) {
            let rel_path = Path::new(tail).join(name);
            let encoded_path = url_encode(&rel_path.display().to_string());
            response.push_str(&format!(
                "&#128196; <a href=\"/{}\">{}</a><br>",
                encoded_path,
                name
            ));
        }
    }

    response.push_str("</body></html>");

    format!("HTTP/1.1 200 OK\r\nContent-Type: text/html\r\n\r\n{}", response).into_bytes()
}

fn handle_request(request: &str) -> Vec<u8> {
    if let Some(current_path) = parse_requested_path(request) {
        if current_path.is_file() {
            return generate_file_response(&current_path);
        } else if current_path.is_dir() {
            // Extract and decode the URL path properly
            let tail = request
                .lines()
                .next()
                .and_then(|line| {
                    let parts: Vec<&str> = line.splitn(3, ' ').collect();
                    if parts.len() >= 2 {
                        Some(parts[1])
                    } else {
                        None
                    }
                })
                .map(|url_path| url_decode(url_path))
                .unwrap_or_default()
                .trim_start_matches('/')
                .to_string();
            return generate_directory_response(&current_path, &tail);
        }
    }
    "HTTP/1.1 400 Bad Request\r\n\r\nBad Request".as_bytes().to_vec()
}

fn main() -> std::io::Result<()> {
    let addr = "127.0.0.1:8080";
    let listener = TcpListener::bind(addr)?;
    println!("Web server running at http://{}/", addr);

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
                    if let Err(_) = stream.write_all(&response) {}
                    if let Err(_) = stream.flush() {}
                });
            }
            Err(_) => {}
        }
    }

    Ok(())
}