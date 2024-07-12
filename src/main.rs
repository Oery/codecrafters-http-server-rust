use std::{env, io::Write};
use tokio::io::AsyncReadExt;

use tokio::{
    io::AsyncWriteExt,
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
    accept_encoding: Option<String>,
    content_encoding: Option<String>,
    content_type: Option<String>,
    content_length: Option<usize>,
    body: Option<String>,
}

impl Request {
    fn parse(request_string: &str) -> std::io::Result<Self> {
        let parts = request_string.split("\r\n").collect::<Vec<&str>>();
        println!("request parts: {:?}", parts);

        let request_line = parts.first().unwrap();
        let method = request_line.split(' ').next().unwrap();
        let path = request_line.split(' ').nth(1).unwrap();

        let body = parts.last().map(|s| s.to_string());

        let mut user_agent = None;
        let mut accept_encoding = None;
        let mut content_encoding = None;
        let mut content_type = None;
        let mut content_length = None;

        let headers = parts.get(1).unwrap();
        println!("headers: {:?}", headers);

        for header in parts {
            if let [key, value] = header.split(": ").collect::<Vec<&str>>()[..] {
                println!("key: {}, value: {}", key, value);
                match key.to_lowercase().as_str() {
                    "user-agent" => user_agent = Some(value.to_string()),
                    "accept-encoding" => accept_encoding = Some(value.to_string()),
                    "content-encoding" => content_encoding = Some(value.to_string()),
                    "content-type" => content_type = Some(value.to_string()),
                    "content-length" => content_length = Some(value.parse::<usize>().unwrap()),
                    _ => {}
                }
            }
        }

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
            accept_encoding,
            content_encoding,
            content_type,
            content_length,
            body,
        })
    }
}

async fn send_text(stream: &mut TcpStream, text: &str) -> std::io::Result<()> {
    let response = format!(
        "HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nContent-Length: {}\r\n\r\n{}",
        text.len(),
        text
    );
    stream.write_all(response.as_bytes()).await
}

async fn send_text_with_encoding(
    stream: &mut TcpStream,
    text: &str,
    encoding: &str,
) -> std::io::Result<()> {
    let response = format!(
        "HTTP/1.1 200 OK\r\nContent-Encoding: {}\r\nContent-Type: text/plain\r\nContent-Length: {}\r\n\r\n{}",
        encoding,
        text.len(),
        text
    );
    stream.write_all(response.as_bytes()).await
}

async fn send_octet_stream(stream: &mut TcpStream, contents: &[u8]) -> std::io::Result<()> {
    let response = format!(
        "HTTP/1.1 200 OK\r\nContent-Type: application/octet-stream\r\nContent-Length: {}\r\n\r\n",
        contents.len(),
    );
    stream.write_all(response.as_bytes()).await?;
    stream.write_all(contents).await?;
    Ok(())
}

async fn handle_connection(mut stream: TcpStream, directory: &str) -> std::io::Result<()> {
    let mut request = [0_u8; 1024];
    let bytes = stream.read(&mut request).await?;
    let request_string = String::from_utf8_lossy(&request[..bytes]).into_owned();

    println!("request: {}", request_string);

    let request = match Request::parse(&request_string) {
        Ok(request) => request,
        Err(e) => {
            let response = format!("HTTP/1.1 400 Bad Request\r\n\r\n{}", e);
            stream.write_all(response.as_bytes()).await?;
            return Ok(());
        }
    };

    println!("{:?}", request);

    let base_route = request.path.split('/').nth(1).unwrap();

    match base_route {
        "" => {
            let response = "HTTP/1.1 200 OK\r\n\r\n";
            stream.write_all(response.as_bytes()).await?;
        }
        "user-agent" => match request.user_agent {
            Some(user_agent) => {
                send_text(&mut stream, &user_agent).await?;
            }
            None => {
                let response = "HTTP/1.1 404 Not Found\r\n\r\n";
                stream.write_all(response.as_bytes()).await?;
            }
        },
        "echo" => match request.path.split('/').nth(2) {
            Some(message) => {
                if let Some(encoding) = request.accept_encoding {
                    send_text_with_encoding(&mut stream, message, &encoding).await?;
                    return Ok(());
                }

                send_text(&mut stream, message).await?;
            }
            None => {
                let response = "HTTP/1.1 404 Not Found\r\n\r\n";
                stream.write_all(response.as_bytes()).await?;
            }
        },
        "files" => match request.method {
            Method::Get => {
                let file = request.path.split('/').nth(2).unwrap();
                let path = format!("{}/{}", directory, file);

                let contents = match std::fs::read(path) {
                    Ok(contents) => contents,
                    Err(_) => {
                        let response = "HTTP/1.1 404 Not Found\r\n\r\n";
                        stream.write_all(response.as_bytes()).await?;
                        return Ok(());
                    }
                };
                send_octet_stream(&mut stream, &contents).await?;
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
                        stream.write_all(response.as_bytes()).await?;
                        return Ok(());
                    }
                };
                file.write_all(content.as_bytes())?;
                let response = "HTTP/1.1 201 Created\r\n\r\n";
                stream.write_all(response.as_bytes()).await?;
            }
        },
        _ => {
            let response = "HTTP/1.1 404 Not Found\r\n\r\n";
            stream.write_all(response.as_bytes()).await?;
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

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let directory = get_directory().unwrap_or("files".into());

    let listener = TcpListener::bind("127.0.0.1:4221").await?;

    loop {
        let directory = directory.clone();
        let (stream, _) = match listener.accept().await {
            Ok(stream) => stream,
            Err(e) => {
                eprintln!("error accepting connection: {}", e);
                continue;
            }
        };
        tokio::spawn(async move { handle_connection(stream, &directory).await });
    }
}
