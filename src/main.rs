use std::{
    env,
    io::{prelude::*, Write},
    net::{TcpListener, TcpStream},
};

#[derive(Debug)]
enum Method {
    Get,
    Post,
}

#[allow(dead_code)]
#[derive(Debug)]
struct Request {
    method: Method,
    path: String,
    user_agent: Option<String>,
    content_type: Option<String>,
    content_length: Option<usize>,
    body: Option<String>,
}

impl Request {
    fn parse(request_string: &str) -> std::io::Result<Self> {
        let parts = request_string.split("\r\n\r\n").collect::<Vec<&str>>();
        println!("{:?}", parts);

        let request_line = parts.first().unwrap();
        let method = request_line.split(' ').next().unwrap();
        let path = request_line.split(' ').nth(1).unwrap();

        let mut user_agent = None;
        let mut content_type = None;
        let mut content_length = None;

        let headers = parts.get(1).unwrap();

        for header in headers.split("\r\n") {
            if let [key, value] = header.split(": ").collect::<Vec<&str>>()[..] {
                match key {
                    "User-Agent" => user_agent = Some(value.to_string()),
                    "Content-Type" => content_type = Some(value.to_string()),
                    "Content-Length" => content_length = Some(value.parse::<usize>().unwrap()),
                    _ => {}
                }
            }
        }

        let body = parts.last().map(|s| s.to_string());

        Ok(Self {
            method: match method {
                "GET" => Method::Get,
                "POST" => Method::Post,
                _ => {
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::Other,
                        format!("unknown method: {}", method),
                    ))
                }
            },
            path: path.to_string(),
            user_agent,
            content_type,
            content_length,
            body,
        })
    }
}

fn send_text(stream: &mut TcpStream, text: &str) -> std::io::Result<()> {
    let response = format!(
        "HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nContent-Length: {}\r\n\r\n{}",
        text.len(),
        text
    );
    stream.write_all(response.as_bytes())
}

fn send_octet_stream(stream: &mut TcpStream, contents: &[u8]) -> std::io::Result<()> {
    let response = format!(
        "HTTP/1.1 200 OK\r\nContent-Type: application/octet-stream\r\nContent-Length: {}\r\n\r\n",
        contents.len(),
    );
    stream.write_all(response.as_bytes())?;
    stream.write_all(contents)?;
    Ok(())
}

fn handle_connection(mut stream: TcpStream, directory: &str) -> std::io::Result<()> {
    let mut request = [0_u8; 1024];
    let bytes = stream.read(&mut request)?;
    let request_string = String::from_utf8_lossy(&request[..bytes]).into_owned();

    let request = Request::parse(&request_string)?;
    println!("{:?}", request);

    let base_route = request.path.split('/').nth(1).unwrap();

    match base_route {
        "" => {
            let response = "HTTP/1.1 200 OK\r\n\r\n";
            stream.write_all(response.as_bytes())?;
        }
        "user-agent" => {
            let user_agent = request.user_agent.unwrap();
            send_text(&mut stream, &user_agent)?;
        }
        "echo" => {
            let message = request.path.split('/').nth(2).unwrap();
            send_text(&mut stream, message)?;
        }
        "files" => match request.method {
            Method::Get => {
                let file = request.path.split('/').nth(2).unwrap();
                let path = format!("{}/{}", directory, file);

                let contents = match std::fs::read(path) {
                    Ok(contents) => contents,
                    Err(_) => {
                        let response = "HTTP/1.1 404 Not Found\r\n\r\n";
                        stream.write_all(response.as_bytes())?;
                        return Ok(());
                    }
                };
                send_octet_stream(&mut stream, &contents)?;
            }
            Method::Post => {
                let file = request.path.split('/').nth(2).unwrap();
                let path = format!("{}/{}", directory, file);

                let body = request.body.unwrap();
                let content = body.split('\r').next().unwrap();

                println!("Saving file to {}", path);

                let mut file = match std::fs::File::create(path) {
                    Ok(file) => file,
                    Err(_) => {
                        let response = "HTTP/1.1 404 Not Found\r\n\r\n";
                        stream.write_all(response.as_bytes())?;
                        return Ok(());
                    }
                };
                file.write_all(content.as_bytes())?;
                let response = "HTTP/1.1 201 Created\r\n\r\n";
                stream.write_all(response.as_bytes())?;
                return Ok(());
            }
        },
        _ => {
            let response = "HTTP/1.1 404 Not Found\r\n\r\n";
            stream.write_all(response.as_bytes())?;
        }
    }

    Ok(())
}

fn get_directory() -> Option<String> {
    let args: Vec<String> = env::args().collect();

    for n in 1..args.len() {
        let arg = args.get(n)?;
        if arg == "--directory" {
            let directory = args.get(n + 1)?;
            println!("Serving files from directory: {}", directory);
            return Some(directory.into());
        }
    }

    None
}

fn main() -> std::io::Result<()> {
    let directory = get_directory().unwrap_or("files".into());

    let listener = TcpListener::bind("127.0.0.1:4221").unwrap();

    for stream in listener.incoming() {
        match stream {
            Ok(_stream) => {
                println!("accepted new connection");

                if let Err(e) = handle_connection(_stream, &directory) {
                    println!("error: {}", e);
                }
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }

    Ok(())
}
