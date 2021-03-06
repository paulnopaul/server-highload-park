use std::{
    io::prelude::*,
    net::TcpListener,
    net::TcpStream,
    fs,
    thread,
    io::{
        BufRead,
        BufReader,
    },
    env,
    path::PathBuf,
    time::SystemTime,
    str,
};
use httpdate;
use urlencoding;

fn main() {
    let host = String::from("0.0.0.0");
    let port = String::from("7878");
    run_server(host, port);
}

fn run_server(host: String, port: String) {
    let address = host + ":" + port.as_str();
    println!("Starting server at: {}", address);
    println!("Root directory: {}", env::current_dir().unwrap().to_str().unwrap());
    let listener = TcpListener::bind(address).unwrap();
    for stream in listener.incoming() {
        let stream = stream.unwrap();
        thread::spawn(|| {
            handle_connection(stream);
        });
    }
    println!("Server shut down");
}

fn handle_connection(mut stream: TcpStream) {
    let mut buffer = [0; 1024];
    stream.read(&mut buffer).unwrap();
    let request_string = str::from_utf8(&buffer).unwrap();
    let (method, request_path) = parse_request_string(request_string);
    let mut root_dir = env::current_dir().unwrap();
    if method != "" {
        handle_request(&mut root_dir, method, request_path, &mut stream);
    }
    stream.flush().unwrap();
}

fn parse_request_string(request_string: &str) -> (String, String) {
    let parsed_request: Vec<&str> = request_string.splitn(3, ' ').collect();
    if parsed_request.len() == 3 {
        let method = parsed_request[0].to_string();
        let request_path = parsed_request[1].trim_start_matches('/');
        let split = request_path.split_once('?');
        let url_path = urlencoding::decode(match split.is_some() {
            true => split.unwrap().0,
            _ => request_path,
        }).unwrap().to_string();
        return (method, url_path);
    }
    return (String::new(), String::new());
}

fn handle_request(root_dir: &mut PathBuf, method: String, request_path: String, stream: &mut TcpStream) {
    let (mut code, res_path) = handle_request_path(root_dir, request_path); // 200, 400, or 403
    if !(method == "GET" || method == "HEAD") {
        code = 405;
    }

    let mut content_type = String::from("");
    let mut content_len: u64 = 0;
    if code == 200 {
        content_len = fs::metadata(&res_path).unwrap().len();
        content_type = get_content_type(res_path.extension().unwrap().to_str().unwrap());
    }
    write_headers(stream, content_len, code, content_type);

    if method == "GET" && code == 200 {
        tcp_write_file(stream, res_path);
    }
}

fn handle_request_path(root_dir: &mut PathBuf, request_path: String) -> (i32, PathBuf) {
    let empty_buf = PathBuf::new();
    let mut full_path = root_dir.join(request_path.as_str());
    let mut add_index = false;

    if full_path.is_dir() {
        full_path = full_path.join("index.html");
        add_index = true;
    }
    if !full_path.exists() {
        if add_index {
            return (403, empty_buf);
        }
        return (404, empty_buf);
    }

    let res_path = fs::canonicalize(full_path).unwrap();
    if !res_path.starts_with(root_dir) {
        return (403, empty_buf);
    }
    return (200, res_path);
}

fn get_content_type(ext: &str) -> String {
    return String::from(match ext {
        "html" => "text/html",
        "css" => "text/css",
        "js" => "text/javascript",
        "jpg" => "image/jpeg",
        "jpeg" => "image/jpeg",
        "png" => "image/png",
        "gif" => "image/gif",
        "swf" => "application/x-shockwave-flash",
        _ => ""
    });
}

fn write_headers(stream: &mut TcpStream, content_len: u64, code: i32, content_type: String) {
    let status_line = format!("HTTP/1.0 {} {}", code, reason_from_code(code));
    let server_line = format!("Server: {}", "rust_static_server");
    let date_line = format!("Date: {}", httpdate::fmt_http_date(SystemTime::now()));
    let connection_line = format!("Connection: {}", "close");
    let content_type_string = match code {
        200 => format!("Content-Type: {}", content_type),
        _ => String::from(""),
    };

    let content_len_line = match code {
        200 => format!("Content-Length: {}", content_len),
        _ => String::from(""),
    };

    let headers = format!(
        "{}\r\n{}\r\n{}\r\n{}\r\n{}\r\n{}\r\n\r\n",
        status_line,
        server_line,
        date_line,
        content_len_line,
        content_type_string,
        connection_line,
    );
    stream.write(headers.as_bytes()).unwrap();
}

fn reason_from_code(code: i32) -> String {
    if code == 200 {
        return String::from("OK");
    } else if code == 404 {
        return String::from("Not Found");
    } else if code == 403 {
        return String::from("Forbidden");
    } else if code == 405 {
        return String::from("Method Not Allowed");
    }
    return String::from("Unknown Code");
}

fn tcp_write_file(stream: &mut TcpStream, path: PathBuf) {
    let mut reader = BufReader::with_capacity(1024 * 128, fs::File::open(&path).unwrap());
    loop {
        let length = {
            let buffer = reader.fill_buf().unwrap();
            let len = buffer.len();
            if len > 0 {
                stream.write(buffer).unwrap();
            }
            len
        };

        if length == 0 {
            break;
        }

        reader.consume(length);
    }
}
