use std::{
    io::prelude::*,
    net::TcpListener,
    net::TcpStream,
    fs,
    thread,
    path::Path,
    io::{
        BufRead,
        BufReader,
    },
    env,
};

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

#[allow(unused_must_use)]
fn handle_connection(mut stream: TcpStream) {
    let mut buffer = [0; 1024];
    stream.read(&mut buffer).unwrap();

    // Parse request
    let request_string = String::from_utf8(buffer.to_vec()).unwrap();
    let res: Vec<&str> = request_string.splitn(3, ' ').collect();
    let method = res[0].to_string();
    let mut path = env::current_dir().unwrap().join(res[1][1..].to_string());

    // Handle request
    let file_exists = path.exists();
    let mut code = 200;
    let mut reason = "OK";
    if !file_exists {
        path = env::current_dir().unwrap().join("static/404.html");
        code = 404;
        reason = "Not Found;"
    }

    let status_line = format!("HTTP/1.0 {} {}", code, reason);

    let content_len = {
        fs::metadata(&path).unwrap().len()
    };

    let response = format!(
        "{}\r\nContent-Length: {}\r\n\r\n",
        status_line,
        content_len,
    );
    stream.write(response.as_bytes()).unwrap();

    if method != "HEAD" {
        // Write file
        let file = fs::File::open(&path).unwrap();
        let mut reader = BufReader::with_capacity(1024 * 128, file);
        loop {
            let length = {
                let buffer = reader.fill_buf().unwrap();
                stream.write(buffer);
                buffer.len()
            };
            if length == 0 {
                break;
            }
            reader.consume(length);
        }
    }

    // End
    stream.flush().unwrap();
}