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
};
use httpdate;

fn main() {
    let host = String::from("127.0.0.1");
    let port = String::from("7878");
    run_server(host, port);
}

fn run_server(host: String, port: String) {
    let address = host + ":" + port.as_str();
    println!("Starting server at: {}", address);
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
    let request_string = String::from_utf8(buffer.to_vec()).unwrap();
    let parsed_request: Vec<&str> = request_string.splitn(3, ' ').collect();
    let method = parsed_request[0].to_string();

    let request_path = parsed_request[1][1..].to_string();
    let mut root_dir = env::current_dir().unwrap();

    handle_request(&mut root_dir, method, request_path, &mut stream);

    stream.flush().unwrap();
}

fn handle_request(root_dir: &mut PathBuf, method: String, request_path: String, stream: &mut TcpStream) {
    let (code, res_path) = handle_request_path(root_dir, request_path); // 200, 400, or 403

    let mut content_len: u64 = 0;
    if code == 200 {
        content_len = fs::metadata(&res_path).unwrap().len()
    }
    write_headers(stream, content_len, code);

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

fn write_headers(stream: &mut TcpStream, content_len: u64, code: i32) {
    let status_line = format!("HTTP/1.0 {} {}", code, reason_from_code(code));
    let server_line = format!("Server: {}", "rust_static_server");
    let date_line = format!("Date: {}", httpdate::fmt_http_date(SystemTime::now()));
    let connection_line = format!("Connection: {}", "close");

    let content_len_line = match code {
        200 => format!("Content-Length: {}", content_len),
        _ => String::from(""),
    };

    let headers = format!(
        "{}\r\n{}\r\n{}\r\n{}\r\n{}\r\n\r\n",
        status_line,
        date_line,
        content_len_line,
        connection_line,
        server_line,
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
