extern crate rs_web_server;
use rs_web_server::ThreadPool;

use std::fs::File;
use std::io::prelude::*;
use std::net::TcpListener;
use std::net::TcpStream;
use time;

//http://127.0.0.1:8080/
fn main() {
    let listener = TcpListener::bind("0.0.0.0:8080").unwrap();
    let pool = ThreadPool::new(8);
    for stream in listener.incoming() {
        let stream = stream.unwrap();
        pool.execute(|| {
            handle_connection(stream);
        });
    }
}

fn get_time() -> String {
    let now = time::now();
    time::strftime("%Y-%m-%d %H:%M:%S", &now).unwrap()
}

fn second_word(s: &String) -> &str {
    let byte = s.as_bytes();
    let mut first: usize = 0;
    let second: usize;
    for (i, &elem) in byte.iter().enumerate() {
        if elem == b' ' {
            if first == 0 {
                first = i;
            } else {
                second = i;
                return &s[(first + 1)..second];
            }
        }
    }
    &s[..]
}

fn get_ext(s: &str) -> &str {
    let byte = s.as_bytes();
    for (i, &elem) in byte.iter().enumerate() {
        if elem == b'.' {
            return &s[(i + 1)..];
        }
    }
    &s[..]
}

fn safe_check(s: &str) -> bool {
    let byte = s.as_bytes();
    if byte.len() == 0 {
        return false;
    }
    let mut prev = byte[0];
    for (_i, &elem) in byte.iter().enumerate() {
        if elem == b'.' {
            if prev == elem {
                return false;
            }
        }
        prev = elem;
    }
    true
}

fn handle_connection(mut stream: TcpStream) {
    let root = "/var/www";
    let mut buffer = [0; 512];
    let client_ip = match stream.peer_addr() {
        Ok(addr) => addr.ip().to_string(),
        Err(_) => String::from("Unknown"),
    };
    let time = get_time();
    stream.read(&mut buffer).unwrap();
    let status_code: u16;
    let buffer_to_s = String::from_utf8_lossy(&buffer[..]).to_string();
    let file_name = second_word(&buffer_to_s);
    if safe_check(&file_name) {
        if file_name == "/" {
            match File::open(format!("{}/index.html", root)) {
                Ok(mut _f) => {
                    let mut contents = String::new();
                    _f.read_to_string(&mut contents).unwrap();
                    status_code = 200;
                    let response = format!("HTTP/1.1 200 OK\r\n\r\n{}", contents);
                    stream.write(response.as_bytes()).unwrap();
                }
                Err(_) => {
                    status_code = 404;
                    stream
                        .write("HTTP/1.1 404 Not FOUND\r\n\r\n{}".as_bytes())
                        .unwrap();
                }
            };
        } else {
            let ext = get_ext(&file_name);
            match File::open(format!("{}{}", root, file_name)) {
                Ok(mut _f) => {
                    if ext == "html" {
                        let mut contents = String::new();
                        _f.read_to_string(&mut contents).unwrap();
                        status_code = 200;
                        let response = format!("HTTP/1.1 200 OK\r\n\r\n{}", contents);
                        stream.write(response.as_bytes()).unwrap();
                    } else {
                        let mut buffer = [0; 65535];
                        status_code = 200;
                        while let std::io::Result::Ok(len) = _f.read(&mut buffer) {
                            if len == 0 {
                                break;
                            } else {
                                match stream.write(&buffer) {
                                    Ok(_f) => {
                                        continue;
                                    }
                                    Err(_e) => {
                                        break;
                                    }
                                };
                            }
                        }
                    }
                }
                Err(_e) => {
                    status_code = 404;
                    stream
                        .write("HTTP/1.1 404 Not FOUND\r\n\r\n{}".as_bytes())
                        .unwrap();
                }
            }
        }
    } else {
        status_code = 403;
        stream
            .write("HTTP/1.1 403 FORBIDDEN\r\n\r\n{}".as_bytes())
            .unwrap();
    }
    println!("{} {} {} GET {}", time, status_code, client_ip, file_name);
    stream.flush().unwrap();
}
